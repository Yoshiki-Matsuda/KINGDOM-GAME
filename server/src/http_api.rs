use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::AppState,
    auth::{self, AuthRequest, AuthResponse},
    model::GameState,
    persistence::save_state,
};

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

pub(crate) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "kingdom-server",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub(crate) async fn api_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "game": "kingdom",
        "mode": "pve",
        "endpoints": { "health": "/health", "ws": "/ws", "api/state": "/api/state" }
    }))
}

type ApiError = (StatusCode, Json<serde_json::Value>);

#[derive(Deserialize)]
pub(crate) struct AdminWipeRequest {
    confirm: Option<String>,
}

pub(crate) async fn auth_register(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let _guard = state.mutation_lock.lock().await;
    let mut game = state.game.write().await;
    let response = auth::register(&state.auth_path, &state.jwt_secret, &mut game, req)
        .await
        .map_err(bad_request)?;
    save_state(&state.state_path, &game).await.map_err(server_error)?;
    let json = serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
    let _ = state.broadcast_tx.send(json);
    Ok(Json(response))
}

pub(crate) async fn auth_login(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let _guard = state.mutation_lock.lock().await;
    let mut game = state.game.write().await;
    let response = auth::login(&state.auth_path, &state.jwt_secret, &mut game, req)
        .await
        .map_err(unauthorized)?;
    save_state(&state.state_path, &game).await.map_err(server_error)?;
    let json = serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
    let _ = state.broadcast_tx.send(json);
    Ok(Json(response))
}

pub(crate) async fn api_state(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<GameState>, ApiError> {
    let _claims = claims_from_headers(&state, &headers)?;
    let game = state.game.read().await;
    Ok(Json(game.clone()))
}

#[derive(Serialize)]
pub(crate) struct WhoamiResponse {
    player_id: String,
    username: String,
}

/// 認証トークンに紐づくプレイヤーID（クライアントは localStorage ではなくここを信頼する）
pub(crate) async fn api_whoami(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WhoamiResponse>, ApiError> {
    let claims = claims_from_headers(&state, &headers)?;
    Ok(Json(WhoamiResponse {
        player_id: claims.sub,
        username: claims.username,
    }))
}

/// ワイプ: ゲームを完全初期化（全マス再生成）。通常の再起動では呼ばない。
pub(crate) async fn admin_wipe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminWipeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let claims = claims_from_headers(&state, &headers)?;
    let admin_player_id = std::env::var("ADMIN_PLAYER_ID").unwrap_or_else(|_| "admin".to_string());
    if claims.sub != admin_player_id {
        return Err(forbidden("管理者権限がありません。"));
    }
    if req.confirm.as_deref() != Some("WIPE") {
        return Err(bad_request("confirm に WIPE を指定してください。".to_string()));
    }
    let _guard = state.mutation_lock.lock().await;
    let new_state = GameState::default();
    {
        let mut game = state.game.write().await;
        *game = new_state.clone();
    }
    if let Err(error) = save_state(&state.state_path, &new_state).await {
        return Err(server_error(error));
    }
    let _ = state.broadcast_tx.send(
        serde_json::to_string(&new_state).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string()),
    );
    Ok(Json(serde_json::json!({ "ok": true, "message": "ワイプしました。" })))
}

fn claims_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<auth::AuthClaims, ApiError> {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(unauthorized("認証が必要です。".to_string()));
    };
    let Ok(value) = value.to_str() else {
        return Err(unauthorized("認証ヘッダーが不正です。".to_string()));
    };
    let Some(token) = auth::bearer_token(value) else {
        return Err(unauthorized("Bearer トークンが必要です。".to_string()));
    };
    auth::verify_token(&state.jwt_secret, token).map_err(unauthorized)
}

fn bad_request(message: String) -> ApiError {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": message })))
}

fn unauthorized(message: String) -> ApiError {
    (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": message })))
}

fn forbidden(message: &str) -> ApiError {
    (StatusCode::FORBIDDEN, Json(serde_json::json!({ "error": message })))
}

fn server_error(error: impl std::fmt::Display) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": error.to_string() })),
    )
}
