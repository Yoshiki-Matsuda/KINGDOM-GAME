use std::path::Path;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::model::{ensure_player_in_game, GameState, TEST_PLAYER_IDS};
use crate::config;
use crate::server_mode::ServerMode;

const TOKEN_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuthUser {
    pub(crate) username: String,
    pub(crate) player_id: String,
    password_hash: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct AuthStore {
    users: Vec<AuthUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuthClaims {
    pub(crate) sub: String,
    pub(crate) username: String,
    pub(crate) mode: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuthRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AuthResponse {
    pub(crate) token: String,
    pub(crate) player_id: String,
    pub(crate) username: String,
}

/// 開発用テストアカウントが未登録なら `data/auth.json` に追加する。
pub(crate) async fn ensure_dev_auth_users(path: &Path) -> Result<(), String> {
    let password = config::dev_auth_password();
    validate_password(&password)?;

    let mut store = load_store(path).await;
    let mut changed = false;
    for &username in TEST_PLAYER_IDS {
        if store.users.iter().any(|user| user.username == username) {
            continue;
        }
        let password_hash = hash_password(&password)?;
        store.users.push(AuthUser {
            username: username.to_string(),
            player_id: username.to_string(),
            password_hash,
        });
        changed = true;
        println!("[kingdom-server] テスト用アカウントを作成しました: {username}（パスワード: {password}）");
    }
    if changed {
        save_store(path, &store).await?;
    }
    Ok(())
}

pub(crate) async fn load_store(path: &Path) -> AuthStore {
    let Ok(data) = tokio::fs::read_to_string(path).await else {
        return AuthStore::default();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

async fn save_store(path: &Path, store: &AuthStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    tokio::fs::write(path, data).await.map_err(|e| e.to_string())
}

pub(crate) async fn register(
    path: &Path,
    jwt_secret: &[u8],
    server_mode: ServerMode,
    game: Option<&mut GameState>,
    req: AuthRequest,
) -> Result<AuthResponse, String> {
    let username = normalize_username(&req.username)?;
    validate_password(&req.password)?;

    let mut store = load_store(path).await;
    if store.users.iter().any(|user| user.username == username) {
        return Err("このユーザー名は既に使われています。".to_string());
    }

    let player_id = username.clone();
    let password_hash = hash_password(&req.password)?;
    store.users.push(AuthUser {
        username: username.clone(),
        player_id: player_id.clone(),
        password_hash,
    });
    save_store(path, &store).await?;

    if let Some(game) = game {
        ensure_player_in_game(game, &player_id)?;
    }
    let token = issue_token(jwt_secret, &player_id, &username, server_mode)?;
    Ok(AuthResponse { token, player_id, username })
}

pub(crate) async fn login(
    path: &Path,
    jwt_secret: &[u8],
    server_mode: ServerMode,
    game: Option<&mut GameState>,
    req: AuthRequest,
) -> Result<AuthResponse, String> {
    let username = normalize_username(&req.username)?;
    let store = load_store(path).await;
    let Some(user) = store.users.iter().find(|user| user.username == username) else {
        return Err("ユーザー名またはパスワードが違います。".to_string());
    };
    verify_password(&req.password, &user.password_hash)?;

    if let Some(game) = game {
        ensure_player_in_game(game, &user.player_id)?;
    }
    let token = issue_token(jwt_secret, &user.player_id, &user.username, server_mode)?;
    Ok(AuthResponse {
        token,
        player_id: user.player_id.clone(),
        username: user.username.clone(),
    })
}

/// 他モードの JWT から、このサーバーの mode 付きトークンを再発行する（HUD 切替用）
pub(crate) async fn exchange_token(
    jwt_secret: &[u8],
    server_mode: ServerMode,
    game: Option<&mut GameState>,
    token: &str,
) -> Result<AuthResponse, String> {
    let claims = verify_token_signature(jwt_secret, token)?;
    if let Some(game) = game {
        ensure_player_in_game(game, &claims.sub)?;
    }
    let new_token = issue_token(jwt_secret, &claims.sub, &claims.username, server_mode)?;
    Ok(AuthResponse {
        token: new_token,
        player_id: claims.sub,
        username: claims.username,
    })
}

fn decode_claims(jwt_secret: &[u8], token: &str) -> Result<AuthClaims, String> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| "認証トークンが無効です。".to_string())
}

/// 署名・有効期限のみ検証（exchange 用。mode は問わない）
pub(crate) fn verify_token_signature(jwt_secret: &[u8], token: &str) -> Result<AuthClaims, String> {
    decode_claims(jwt_secret, token)
}

/// このサーバーの mode と一致するトークンのみ許可
pub(crate) fn verify_token_for_mode(
    jwt_secret: &[u8],
    token: &str,
    expected_mode: ServerMode,
) -> Result<AuthClaims, String> {
    let claims = decode_claims(jwt_secret, token)?;
    if claims.mode != expected_mode.as_str() {
        return Err("このトークンは別モード用です。".to_string());
    }
    Ok(claims)
}

pub(crate) fn bearer_token(value: &str) -> Option<&str> {
    value.strip_prefix("Bearer ").filter(|token| !token.trim().is_empty())
}

fn issue_token(
    jwt_secret: &[u8],
    player_id: &str,
    username: &str,
    server_mode: ServerMode,
) -> Result<String, String> {
    let exp = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .saturating_add(TOKEN_TTL_SECONDS) as usize;
    let claims = AuthClaims {
        sub: player_id.to_string(),
        username: username.to_string(),
        mode: server_mode.as_str().to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret),
    )
    .map_err(|e| e.to_string())
}

fn normalize_username(raw: &str) -> Result<String, String> {
    let username = raw.trim().to_ascii_lowercase();
    if username.len() < 3 || username.len() > 32 {
        return Err("ユーザー名は3〜32文字にしてください。".to_string());
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("ユーザー名に使える文字は英数字、_、- です。".to_string());
    }
    Ok(username)
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("パスワードは8文字以上にしてください。".to_string());
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), String> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| {
        "保存されているパスワード情報が壊れています。".to_string()
    })?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "ユーザー名またはパスワードが違います。".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secret() -> Vec<u8> {
        b"test-secret-for-jwt-mode".to_vec()
    }

    #[test]
    fn issue_and_verify_mode_bound_token() {
        let token = issue_token(&secret(), "player_a", "player_a", ServerMode::Pve).unwrap();
        let ok = verify_token_for_mode(&secret(), &token, ServerMode::Pve).unwrap();
        assert_eq!(ok.sub, "player_a");
        assert_eq!(ok.mode, "pve");
        assert!(verify_token_for_mode(&secret(), &token, ServerMode::Pvp).is_err());
    }

    #[test]
    fn exchange_accepts_cross_mode_signature() {
        let pve_token = issue_token(&secret(), "u1", "u1", ServerMode::Pve).unwrap();
        let claims = verify_token_signature(&secret(), &pve_token).unwrap();
        assert_eq!(claims.mode, "pve");
        let pvp_token = issue_token(&secret(), &claims.sub, &claims.username, ServerMode::Pvp).unwrap();
        verify_token_for_mode(&secret(), &pvp_token, ServerMode::Pvp).unwrap();
    }
}
