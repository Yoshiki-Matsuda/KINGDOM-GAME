use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    app_state::{AppState, GameStore},
    auth::{self, AuthRequest, AuthResponse},
    model::{client_view_state, GameState},
    persistence,
    db::world_repo,
};

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    mode: &'static str,
    world_cols: u16,
    world_rows: u16,
}

pub(crate) async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "kingdom-server",
        version: env!("CARGO_PKG_VERSION"),
        mode: state.server_mode.as_str(),
        world_cols: state.world_config.cols,
        world_rows: state.world_config.rows,
    })
}

pub(crate) async fn api_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "game": "kingdom",
        "mode": state.server_mode.as_str(),
        "world_cols": state.world_config.cols,
        "world_rows": state.world_config.rows,
        "endpoints": {
            "health": "/health",
            "ws": "/ws",
            "api/state": "/api/state",
            "auth/exchange": "/auth/exchange"
        }
    }))
}

type ApiError = (StatusCode, Json<serde_json::Value>);

#[derive(Deserialize)]
pub(crate) struct AdminWipeRequest {
    confirm: Option<String>,
    terrain_seed: Option<u64>,
}

pub(crate) async fn auth_register(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let _guard = state.mutation_lock.lock().await;
    let server_mode = state.server_mode;
    let response = match &state.store {
        GameStore::Shared(game) => {
            let mut game = game.write().await;
            let response = auth::register(
                &state.db_pool,
                &state.jwt_secret,
                server_mode,
                Some(&mut game),
                req,
            )
            .await
            .map_err(bad_request)?;
            persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &game).await.map_err(server_error)?;
            let json =
                serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
            state.broadcast_json(None, json);
            response
        }
        GameStore::PerPlayer(_) => {
            auth::register(&state.db_pool, &state.jwt_secret, server_mode, None, req)
                .await
                .map_err(bad_request)?
        }
    };
    Ok(Json(response))
}

pub(crate) async fn auth_login(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let _guard = state.mutation_lock.lock().await;
    let server_mode = state.server_mode;
    let response = match &state.store {
        GameStore::Shared(game) => {
            let mut game = game.write().await;
            let response = auth::login(
                &state.db_pool,
                &state.jwt_secret,
                server_mode,
                Some(&mut game),
                req,
            )
            .await
            .map_err(unauthorized)?;
            persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &game).await.map_err(server_error)?;
            let json =
                serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
            state.broadcast_json(None, json);
            response
        }
        GameStore::PerPlayer(_) => {
            auth::login(&state.db_pool, &state.jwt_secret, server_mode, None, req)
                .await
                .map_err(unauthorized)?
        }
    };
    Ok(Json(response))
}

pub(crate) async fn auth_exchange(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, ApiError> {
    let token = bearer_from_headers(&headers)?;
    let _guard = state.mutation_lock.lock().await;
    let server_mode = state.server_mode;
    let response = match &state.store {
        GameStore::Shared(game) => {
            let mut game = game.write().await;
            let response = auth::exchange_token(
                &state.db_pool,
                &state.jwt_secret,
                server_mode,
                Some(&mut game),
                token,
            )
            .await
            .map_err(bad_request)?;
            persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &game).await.map_err(server_error)?;
            let json =
                serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string());
            state.broadcast_json(None, json);
            response
        }
        GameStore::PerPlayer(_) => {
            auth::exchange_token(&state.db_pool, &state.jwt_secret, server_mode, None, token)
                .await
                .map_err(bad_request)?
        }
    };
    Ok(Json(response))
}

pub(crate) async fn api_state(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<GameState>, ApiError> {
    let claims = claims_from_headers(&state, &headers)?;
    match &state.store {
        GameStore::Shared(game) => {
            let game = game.read().await;
            Ok(Json(client_view_state(
                &game,
                &claims.sub,
                state.server_mode,
            )))
        }
        GameStore::PerPlayer(mgr) => {
            let world = mgr.get_or_create_world(&claims.sub).await;
            let game = world.read().await;
            Ok(Json(client_view_state(
                &game,
                &claims.sub,
                state.server_mode,
            )))
        }
    }
}

#[derive(Serialize)]
pub(crate) struct WhoamiResponse {
    player_id: String,
    username: String,
}

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

pub(crate) async fn admin_wipe(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<AdminWipeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let claims = claims_from_headers(&state, &headers)?;
    let admin_player_id = crate::config::admin_player_id();
    if claims.sub != admin_player_id {
        return Err(forbidden("管理者権限がありません。"));
    }
    if req.confirm.as_deref() != Some("WIPE") {
        return Err(bad_request("confirm に WIPE を指定してください。".to_string()));
    }
    let _guard = state.mutation_lock.lock().await;

    match &state.store {
        GameStore::Shared(game) => {
            let new_state = GameState::default();
            {
                let mut game = game.write().await;
                *game = new_state.clone();
            }
            let _ = world_repo::delete_world(&state.db_pool, state.pvp_world_id()).await;
            persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &new_state).await.map_err(server_error)?;
            state.broadcast_json(
                None,
                serde_json::to_string(&new_state).unwrap_or_default(),
            );
        }
        GameStore::PerPlayer(mgr) => {
            let world = mgr.get_or_create_world(&claims.sub).await;
            let mut world_config = state.world_config;
            world_config.terrain_seed = crate::model::resolve_terrain_seed(req.terrain_seed);
            let new_state = crate::pve_world::new_pve(&claims.sub, world_config);
            {
                let mut game = world.write().await;
                *game = new_state.clone();
            }
            mgr.save_world(&claims.sub, &new_state).await;
            state.broadcast_json(
                Some(&claims.sub),
                serde_json::to_string(&new_state).unwrap_or_default(),
            );
        }
    }

    Ok(Json(serde_json::json!({ "ok": true, "message": "ワイプしました。" })))
}

fn bearer_from_headers(headers: &HeaderMap) -> Result<&str, ApiError> {
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return Err(unauthorized("認証が必要です。".to_string()));
    };
    let Ok(value) = value.to_str() else {
        return Err(unauthorized("認証ヘッダーが不正です。".to_string()));
    };
    auth::bearer_token(value).ok_or_else(|| unauthorized("Bearer トークンが必要です。".to_string()))
}

fn claims_from_headers(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<auth::AuthClaims, ApiError> {
    let token = bearer_from_headers(headers)?;
    auth::verify_token_for_mode(&state.jwt_secret, token, state.server_mode).map_err(unauthorized)
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
