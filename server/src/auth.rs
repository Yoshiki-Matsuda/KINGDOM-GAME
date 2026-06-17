use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;

use crate::model::{ensure_player_in_game, GameState, TEST_PLAYER_IDS};
use crate::config;
use crate::db::auth_repo;
use crate::server_mode::ServerMode;

const TOKEN_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

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

/// 開発用テストアカウントをDBに確保する
pub(crate) async fn ensure_dev_auth_users(pool: &SqlitePool) -> Result<(), String> {
    let password = config::dev_auth_password();
    validate_password(&password)?;
    let password_hash = hash_password(&password)?;
    auth_repo::ensure_dev_users(pool, &password_hash, TEST_PLAYER_IDS).await
}

pub(crate) async fn register(
    pool: &SqlitePool,
    jwt_secret: &[u8],
    server_mode: ServerMode,
    game: Option<&mut GameState>,
    req: AuthRequest,
) -> Result<AuthResponse, String> {
    let username = normalize_username(&req.username)?;
    validate_password(&req.password)?;

    if auth_repo::find_user_by_username(pool, &username).await.is_some() {
        return Err("このユーザー名は既に使われています。".to_string());
    }

    let player_id = username.clone();
    let password_hash = hash_password(&req.password)?;
    let user = auth_repo::DbAuthUser {
        username: username.clone(),
        player_id: player_id.clone(),
        password_hash,
    };
    auth_repo::insert_user(pool, &user).await.map_err(|e| e.to_string())?;

    if let Some(game) = game {
        ensure_player_in_game(game, &player_id)?;
    }
    let token = issue_token(jwt_secret, &player_id, &username, server_mode)?;
    Ok(AuthResponse { token, player_id, username })
}

pub(crate) async fn login(
    pool: &SqlitePool,
    jwt_secret: &[u8],
    server_mode: ServerMode,
    game: Option<&mut GameState>,
    req: AuthRequest,
) -> Result<AuthResponse, String> {
    let username = normalize_username(&req.username)?;
    let user = auth_repo::find_user_by_username(pool, &username).await
        .ok_or_else(|| "ユーザー名またはパスワードが違います。".to_string())?;
    verify_password(&req.password, &user.password_hash)?;

    if let Some(game) = game {
        ensure_player_in_game(game, &user.player_id)?;
    }
    let token = issue_token(jwt_secret, &user.player_id, &user.username, server_mode)?;
    Ok(AuthResponse {
        token,
        player_id: user.player_id,
        username: user.username,
    })
}

/// 他モードの JWT から、このサーバーの mode 付きトークンを再発行する（HUD 切替用）
pub(crate) async fn exchange_token(
    _pool: &SqlitePool,
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
