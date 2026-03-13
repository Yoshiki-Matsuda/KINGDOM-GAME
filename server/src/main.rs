//! キングダム戦略ゲーム — Rust バックエンド
//!
//! - HTTP: /health, /api, /api/state, POST /admin/wipe（ワイプ時のみ完全初期化）
//! - WebSocket: /ws — 接続時に状態送信、行動メッセージ受信で状態更新し全クライアントに配信
//! - 永続化: 起動時に data/state.json をロード、行動ごとに保存。サーバー再起動で状態復元。

mod app_state;
mod cards;
mod facilities;
mod http_api;
mod items;
mod model;
mod model_actions;
mod model_ruins;
mod persistence;
mod realtime;
mod ruins;
mod ruin_scheduler;
mod skills;

use axum::{routing::{get, post}, Router};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use app_state::AppState;
use model::{cleanup_expired_ruins, GameState};
use persistence::{load_state, save_state, state_path};
use ruin_scheduler::spawn_ruin_scheduler;

// ---------- main ----------

#[tokio::main]
async fn main() {
    let state_path = state_path();
    let mut game = match load_state(&state_path).await {
        Some(loaded) => {
            println!("[kingdom-server] 保存済み状態を読み込みました: {}", state_path.display());
            loaded
        }
        None => {
            let default = GameState::default();
            let _ = save_state(&state_path, &default).await;
            println!("[kingdom-server] 新規ゲームを初期化し保存しました: {}", state_path.display());
            default
        }
    };
    
    // 起動時に期限切れの遺跡をクリーンアップ
    if cleanup_expired_ruins(&mut game) {
        let _ = save_state(&state_path, &game).await;
        println!("[kingdom-server] 期限切れの遺跡をクリーンアップしました");
    }

    let dev_auto_win = std::env::var("DEV_AUTO_WIN")
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if dev_auto_win {
        println!("[kingdom-server] DEV_AUTO_WIN=1: 攻撃側10倍有利（ローカル開発モード）");
    }

    let (broadcast_tx, _) = broadcast::channel(32);
    let app_state = AppState {
        game: Arc::new(RwLock::new(game)),
        broadcast_tx,
        state_path,
        dev_auto_win,
    };

    // 遺跡スポーン用のバックグラウンドタスク（Routerに渡す前にclone）
    let ruin_state = app_state.clone();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(http_api::health))
        .route("/api", get(http_api::api_info))
        .route("/api/state", get(http_api::api_state))
        .route("/admin/wipe", post(http_api::admin_wipe))
        .route("/ws", get(realtime::ws_handler))
        .layer(cors)
        .with_state(app_state);
    spawn_ruin_scheduler(ruin_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("[kingdom-server] listening on http://{}", addr);
    println!("  GET /health      -> ヘルスチェック");
    println!("  GET /api/state   -> 現在のゲーム状態（JSON）");
    println!("  POST /admin/wipe -> ワイプ（完全初期化。メンテ再起動では使わない）curl -X POST http://127.0.0.1:3000/admin/wipe");
    println!("  GET /ws          -> WebSocket（状態配信・行動受付）");
    println!("  行動例: 送信 {{\"action\":\"end_turn\"}} でターン進行");
    println!("  遺跡: 60秒ごとにスポーン判定、最大3個");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
