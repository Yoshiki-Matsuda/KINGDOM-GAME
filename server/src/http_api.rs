use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::{
    app_state::AppState,
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

pub(crate) async fn api_state(State(state): State<AppState>) -> Json<GameState> {
    let game = state.game.read().await;
    Json(game.clone())
}

/// ワイプ: ゲームを完全初期化（全マス再生成）。通常の再起動では呼ばない。
pub(crate) async fn admin_wipe(State(state): State<AppState>) -> Json<serde_json::Value> {
    let new_state = GameState::default();
    {
        let mut game = state.game.write().await;
        *game = new_state.clone();
    }
    if let Err(error) = save_state(&state.state_path, &new_state).await {
        return Json(serde_json::json!({ "ok": false, "error": error.to_string() }));
    }
    let _ = state.broadcast_tx.send(
        serde_json::to_string(&new_state).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string()),
    );
    Json(serde_json::json!({ "ok": true, "message": "ワイプしました。" }))
}
