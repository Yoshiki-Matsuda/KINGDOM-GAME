use std::path::Path;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::model::{ensure_player_in_game, GameState};

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
    game: &mut GameState,
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

    ensure_player_in_game(game, &player_id);
    let token = issue_token(jwt_secret, &player_id, &username)?;
    Ok(AuthResponse { token, player_id, username })
}

pub(crate) async fn login(
    path: &Path,
    jwt_secret: &[u8],
    game: &mut GameState,
    req: AuthRequest,
) -> Result<AuthResponse, String> {
    let username = normalize_username(&req.username)?;
    let store = load_store(path).await;
    let Some(user) = store.users.iter().find(|user| user.username == username) else {
        return Err("ユーザー名またはパスワードが違います。".to_string());
    };
    verify_password(&req.password, &user.password_hash)?;

    ensure_player_in_game(game, &user.player_id);
    let token = issue_token(jwt_secret, &user.player_id, &user.username)?;
    Ok(AuthResponse {
        token,
        player_id: user.player_id.clone(),
        username: user.username.clone(),
    })
}

pub(crate) fn verify_token(jwt_secret: &[u8], token: &str) -> Result<AuthClaims, String> {
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

pub(crate) fn bearer_token(value: &str) -> Option<&str> {
    value.strip_prefix("Bearer ").filter(|token| !token.trim().is_empty())
}

fn issue_token(jwt_secret: &[u8], player_id: &str, username: &str) -> Result<String, String> {
    let exp = time::OffsetDateTime::now_utc()
        .unix_timestamp()
        .saturating_add(TOKEN_TTL_SECONDS) as usize;
    let claims = AuthClaims {
        sub: player_id.to_string(),
        username: username.to_string(),
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
