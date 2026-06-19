//! キングダム戦略ゲーム — Rust バックエンド
//!
//! - HTTP: /health, /api, /api/state, POST /admin/wipe
//! - WebSocket: /ws — 接続時に状態送信、行動メッセージ受信で状態更新
//! - PVP: 共有ワールド (SERVER_MODE=pvp, 既定ポート3000)
//! - PVE: プレイヤー別ワールド (SERVER_MODE=pve, 既定ポート3001)

mod ai_actions;
mod ai_kingdom_scheduler;
mod app_state;
mod auth;
mod config;
mod db;
mod dev_bot;
mod cards;
mod facilities;
mod http_api;
mod items;
mod model;
mod model_actions;
mod model_ruins;
mod paths;
mod persistence;
mod pve_world;
mod realtime;
mod ruins;
mod server_mode;
mod skills;
mod world_manager;
mod world_scheduler;
mod march_scheduler;

use axum::{routing::{get, post}, Router};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, Mutex, Notify, RwLock};
use app_state::{AppState, GameStore};
use db::world_repo;
use model::{cleanup_expired_ruins, ensure_player_in_game, GameState, TEST_PLAYER_IDS, WorldConfig};
use paths::project_root;
use server_mode::ServerMode;
use world_manager::{spawn_world_eviction, WorldManager};
use world_scheduler::spawn_world_scheduler;
use march_scheduler::spawn_march_scheduler;

#[tokio::main]
async fn main() {
    let server_mode = ServerMode::from_env();
    let world_config = WorldConfig::from_env();

    println!(
        "[kingdom-server] project_root={}",
        project_root().display()
    );

    // DB接続（SQLite）
    let database_url = config::database_url();
    println!("[kingdom-server] DATABASE_URL={}", database_url);
    let pool = db::create_pool(&database_url).await.unwrap_or_else(|e| {
        eprintln!("[kingdom-server] DB接続失敗: {e}");
        std::process::exit(1);
    });

    let store = match server_mode {
        ServerMode::Pvp => {
            let pvp_id = world_repo::pvp_world_id();
            let mut game = match persistence::load_state(&pool, pvp_id).await {
                Some(loaded) => {
                    println!("[kingdom-server] PVP保存済み状態を読み込みました(DB)");
                    loaded
                }
                None => {
                    let mut default = GameState::default();
                    default.world = world_config;
                    let _ = persistence::save_state(&pool, pvp_id, "pvp", &default).await;
                    println!("[kingdom-server] PVP新規ゲームを初期化(DB)");
                    default
                }
            };

            model::migrate_legacy_neutral_enemies(&mut game);
            let _ = crate::items::migrate_inventory_gold_to_resources(&mut game);
            for &player_id in TEST_PLAYER_IDS {
                if let Err(e) = ensure_player_in_game(&mut game, player_id) {
                    eprintln!(
                        "[kingdom-server] 警告: {player_id} をゲームに追加できません: {e}"
                    );
                }
            }
            model::refresh_all_test_players(&mut game);
            let _ = persistence::save_state(&pool, pvp_id, "pvp", &game).await;

            if cleanup_expired_ruins(&mut game) {
                let _ = persistence::save_state(&pool, pvp_id, "pvp", &game).await;
                println!("[kingdom-server] 期限切れの遺跡をクリーンアップしました");
            }

            GameStore::Shared(Arc::new(RwLock::new(game)))
        }
        ServerMode::Pve => {
            println!("[kingdom-server] PVEモード: ワールドは接続時に生成(DB)");
            let mgr = Arc::new(WorldManager::new(pool.clone(), world_config));
            spawn_world_eviction(mgr.clone());
            GameStore::PerPlayer(mgr)
        }
    };

    if let Err(e) = auth::ensure_dev_auth_users(&pool).await {
        eprintln!("[kingdom-server] テスト用アカウントの初期化に失敗: {e}");
        std::process::exit(1);
    }

    let dev_auto_win = config::dev_auto_win_enabled();
    if dev_auto_win {
        match server_mode {
            ServerMode::Pvp => println!(
                "[kingdom-server] DEV_AUTO_WIN=1: 攻撃ダメージ10倍 + 所持魔獣スタミナ無限 + 敵BOT（player）が既存WS経由で自動攻撃"
            ),
            ServerMode::Pve => println!(
                "[kingdom-server] DEV_AUTO_WIN=1: 攻撃ダメージ10倍 + 所持魔獣スタミナ無限"
            ),
        }
    }

    let (broadcast_tx, _) = broadcast::channel(256);
    let jwt_secret = config::jwt_secret_bytes();
    let app_state = AppState {
        server_mode,
        store,
        mutation_lock: Arc::new(Mutex::new(())),
        broadcast_tx,
        db_pool: pool,
        jwt_secret: Arc::new(jwt_secret),
        dev_auto_win,
        world_config,
        march_wake: Arc::new(Notify::new()),
    };

    spawn_world_scheduler(app_state.clone());
    spawn_march_scheduler(app_state.clone());

    if server_mode == ServerMode::Pve {
        ai_kingdom_scheduler::spawn_ai_kingdom_scheduler(app_state.clone());
    }

    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/health", get(http_api::health))
        .route("/api", get(http_api::api_info))
        .route("/api/state", get(http_api::api_state))
        .route("/api/whoami", get(http_api::api_whoami))
        .route("/auth/register", post(http_api::auth_register))
        .route("/auth/login", post(http_api::auth_login))
        .route("/auth/exchange", post(http_api::auth_exchange))
        .route("/admin/wipe", post(http_api::admin_wipe))
        .route("/ws", get(realtime::ws_handler))
        .layer(cors)
        .with_state(app_state.clone());

    let port = config::listen_port(server_mode);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!(
        "[kingdom-server] mode={} listening on http://{}",
        server_mode.as_str(),
        addr
    );
    println!("  GET /health      -> ヘルスチェック");
    println!("  GET /api/state   -> 現在のゲーム状態（JSON）");
    println!(
        "  POST /admin/wipe -> ワイプ curl -X POST http://127.0.0.1:{}/admin/wipe",
        port
    );
    println!("  GET /ws          -> WebSocket（状態配信・行動受付）");
    println!("  遺跡: 60秒ごとにスポーン判定、最大3個/ワールド");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            eprintln!("[kingdom-server] エラー: {} は既に使用中です。", addr);
            std::process::exit(1);
        }
        Err(e) => panic!("bind: {e}"),
    };

    if server_mode == ServerMode::Pvp {
        if let Some(bot_config) = dev_bot::DevBotConfig::from_env(port) {
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                dev_bot::run(bot_config).await;
            });
        }
    }

    axum::serve(listener, app).await.expect("serve");
}
