//! キングダム戦略ゲーム — Rust バックエンド
//!
//! - HTTP: /health, /api, /api/state, POST /admin/wipe（ワイプ時のみ完全初期化）
//! - WebSocket: /ws — 接続時に状態送信、行動メッセージ受信で状態更新し全クライアントに配信
//! - 永続化: 起動時に data/state.json をロード、行動ごとに保存。サーバー再起動で状態復元。

mod app_state;
mod auth;
mod dev_bot;
mod game_log;
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
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::cors::{Any, CorsLayer};
use app_state::AppState;
use model::{cleanup_expired_ruins, ensure_player_in_game, GameState, TEST_PLAYER_IDS};
use persistence::{load_state, save_state, state_path};
use ruin_scheduler::spawn_ruin_scheduler;

// ---------- main ----------

#[tokio::main]
async fn main() {
    let state_path = state_path();
    let auth_path = std::path::PathBuf::from("data/auth.json");
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

    model::migrate_log_timestamps(&mut game);
    model::migrate_legacy_neutral_enemies(&mut game);
    for &player_id in TEST_PLAYER_IDS {
        ensure_player_in_game(&mut game, player_id);
    }
    model::refresh_all_test_players(&mut game);

    let _ = save_state(&state_path, &game).await;

    if let Err(e) = auth::ensure_dev_auth_users(&auth_path).await {
        eprintln!("[kingdom-server] テスト用アカウントの初期化に失敗: {e}");
        std::process::exit(1);
    }

    // 起動時に期限切れの遺跡をクリーンアップ
    if cleanup_expired_ruins(&mut game) {
        let _ = save_state(&state_path, &game).await;
        println!("[kingdom-server] 期限切れの遺跡をクリーンアップしました");
    }

    let dev_auto_win = std::env::var("DEV_AUTO_WIN")
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    if dev_auto_win {
        println!(
            "[kingdom-server] DEV_AUTO_WIN=1: 攻撃側10倍有利 + 敵BOT（player）が既存WS経由で自動攻撃"
        );
    }

    // 戦闘処理中にブロードキャストが留まりすぎると RecvError::Lagged になるため余裕を持たせる
    let (broadcast_tx, _) = broadcast::channel(256);
    let jwt_secret = std::env::var("AUTH_JWT_SECRET")
        .unwrap_or_else(|_| "dev-only-change-this-secret".to_string())
        .into_bytes();
    let app_state = AppState {
        game: Arc::new(RwLock::new(game)),
        mutation_lock: Arc::new(Mutex::new(())),
        broadcast_tx,
        state_path,
        auth_path,
        jwt_secret: Arc::new(jwt_secret),
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
        .route("/api/whoami", get(http_api::api_whoami))
        .route("/auth/register", post(http_api::auth_register))
        .route("/auth/login", post(http_api::auth_login))
        .route("/admin/wipe", post(http_api::admin_wipe))
        .route("/ws", get(realtime::ws_handler))
        .layer(cors)
        .with_state(app_state);
    spawn_ruin_scheduler(ruin_state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("[kingdom-server] listening on http://{}", addr);
    println!("  GET /health      -> ヘルスチェック");
    println!("  GET /api/state   -> 現在のゲーム状態（JSON）");
    println!(
        "  POST /admin/wipe -> ワイプ（完全初期化。メンテ再起動では使わない）curl -X POST http://127.0.0.1:{}/admin/wipe",
        port
    );
    println!(
        "  テスト用アカウント: offline_test（人間）/ player（敵BOT）パスワード test12345"
    );
    println!(
        "  ローカル開発: DEV_AUTO_WIN=1 cargo run（攻撃有利+敵BOT自動起動）/ BOTのみ: DEV_BOT=1 cargo run"
    );
    println!("  GET /ws          -> WebSocket（状態配信・行動受付）");
    println!("  行動例: 送信 {{\"action\":\"end_turn\"}} でターン進行");
    println!("  遺跡: 60秒ごとにスポーン判定、最大3個");
    println!("  （ポートが使用中なら: 別ターミナルのサーバーを止めるか PORT=3001 などで起動）");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            eprintln!(
                "[kingdom-server] エラー: {} は既に使用中です（別の kingdom-server や他プロセスを終了するか PORT 環境変数で別ポートを指定してください）。",
                addr
            );
            std::process::exit(1);
        }
        Err(e) => panic!("bind: {e}"),
    };

    if let Some(bot_config) = dev_bot::DevBotConfig::from_env(port) {
        println!(
            "  敵BOT: {} → {}（既存 /auth/login + /ws）",
            bot_config.username, bot_config.target_player
        );
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            dev_bot::run(bot_config).await;
        });
    }

    axum::serve(listener, app).await.expect("serve");
}
