//! キングダム戦略ゲーム — Rust バックエンド
//!
//! - HTTP: /health, /api, /api/state, POST /admin/wipe（ワイプ時のみ完全初期化）
//! - WebSocket: /ws — 接続時に状態送信、行動メッセージ受信で状態更新し全クライアントに配信
//! - 永続化: 起動時に data/state.json をロード、行動ごとに保存。サーバー再起動で状態復元。

mod model;
mod skills;
mod items;
mod ruins;
mod facilities;
mod cards;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::{get, post},
    Router,
    Json,
};
use model::{apply_action, cleanup_expired_ruins, spawn_random_ruin, count_ruins, Action, GameState};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

/// 状態ファイル（サーバー起動時の CWD 基準）
const STATE_FILE: &str = "data/state.json";

// ---------- 共有状態 ----------

#[derive(Clone)]
struct AppState {
    game: Arc<RwLock<GameState>>,
    broadcast_tx: broadcast::Sender<String>,
    state_path: PathBuf,
    /// ローカル開発用: true のとき攻撃側10倍有利
    dev_auto_win: bool,
}

// ---------- HTTP ----------

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "kingdom-server",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn api_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "game": "kingdom",
        "mode": "pve",
        "endpoints": { "health": "/health", "ws": "/ws", "api/state": "/api/state" }
    }))
}

async fn api_state(State(state): State<AppState>) -> Json<GameState> {
    let g = state.game.read().await;
    Json(g.clone())
}

/// ワイプ: ゲームを完全初期化（全マス再生成）。通常の再起動では呼ばない。
async fn admin_wipe(State(state): State<AppState>) -> Json<serde_json::Value> {
    let new_state = GameState::default();
    {
        let mut g = state.game.write().await;
        *g = new_state.clone();
    }
    if let Err(e) = save_state(&state.state_path, &new_state).await {
        return Json(serde_json::json!({ "ok": false, "error": e.to_string() }));
    }
    let _ = state.broadcast_tx.send(
        serde_json::to_string(&new_state).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string()),
    );
    Json(serde_json::json!({ "ok": true, "message": "ワイプしました。" }))
}

async fn load_state(path: &std::path::Path) -> Option<GameState> {
    let data = tokio::fs::read_to_string(path).await.ok()?;
    serde_json::from_str(&data).ok()
}

async fn save_state(path: &std::path::Path, state: &GameState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let data = serde_json::to_string_pretty(state)?;
    tokio::fs::write(path, data).await?;
    Ok(())
}

// ---------- WebSocket ----------

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // 接続時に期限切れの遺跡をクリーンアップしてから状態を送信
    let json = {
        let mut g = state.game.write().await;
        if cleanup_expired_ruins(&mut g) {
            let _ = save_state(&state.state_path, &g).await;
        }
        serde_json::to_string(&*g).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
    };
    if socket.send(Message::Text(json)).await.is_err() {
        return;
    }

    let mut broadcast_rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                let Ok(text) = msg.to_text() else { continue };
                let Ok(action) = serde_json::from_str::<Action>(text) else { continue };
                let new_state = {
                    let g = state.game.read().await;
                    let out = apply_action(&g, &action, state.dev_auto_win);
                    if let Action::Attack { to_territory_id, .. } = &action {
                        let conquered = out.territories.iter().find(|t| t.id == *to_territory_id).map(|t| t.owner_id.as_deref() == Some("player")).unwrap_or(false);
                        println!("[kingdom-server] 攻撃処理: to={} conquered={}", to_territory_id, conquered);
                    }
                    out
                };
                {
                    let mut g = state.game.write().await;
                    *g = new_state.clone();
                }
                let _ = save_state(&state.state_path, &new_state).await;
                let json = serde_json::to_string(&new_state).unwrap_or_default();
                let _ = state.broadcast_tx.send(json.clone());
                // 行動を送ったクライアントに直接返す（確実に届ける）
                let _ = socket.send(Message::Text(json)).await;
            }
            Ok(json) = broadcast_rx.recv() => {
                if socket.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
            else => break,
        }
    }
}

// ---------- main ----------

#[tokio::main]
async fn main() {
    let state_path = PathBuf::from(STATE_FILE);
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
        .route("/health", get(health))
        .route("/api", get(api_info))
        .route("/api/state", get(api_state))
        .route("/admin/wipe", post(admin_wipe))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(app_state);
    tokio::spawn(async move {
        use rand::Rng;
        
        // 遺跡の最大数
        const MAX_RUINS: usize = 3;
        // スポーン判定間隔（秒）
        const SPAWN_INTERVAL_SECS: u64 = 60;
        // スポーン確率（30%）
        const SPAWN_CHANCE: f64 = 0.30;
        
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(SPAWN_INTERVAL_SECS)).await;
            
            let mut changed = false;
            {
                let mut g = ruin_state.game.write().await;
                
                // 期限切れの遺跡をクリーンアップ
                if cleanup_expired_ruins(&mut g) {
                    changed = true;
                    println!("[kingdom-server] 期限切れの遺跡をクリーンアップしました");
                }
                
                // 遺跡が最大数未満、かつ確率判定に成功したら新しい遺跡をスポーン
                let current_count = count_ruins(&g);
                if current_count < MAX_RUINS {
                    let roll: f64 = rand::thread_rng().gen();
                    if roll < SPAWN_CHANCE {
                        if spawn_random_ruin(&mut g) {
                            changed = true;
                            println!("[kingdom-server] 新しい遺跡が出現しました！ (現在: {}個)", current_count + 1);
                        }
                    }
                }
            }
            
            if changed {
                // 状態を保存して全クライアントに配信
                let g = ruin_state.game.read().await;
                let _ = save_state(&ruin_state.state_path, &g).await;
                let json = serde_json::to_string(&*g).unwrap_or_default();
                let _ = ruin_state.broadcast_tx.send(json);
            }
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("[kingdom-server] listening on http://{}", addr);
    println!("  GET /health      -> ヘルスチェック");
    println!("  GET /api/state   -> 現在のゲーム状態（JSON）");
    println!("  POST /admin/wipe -> ワイプ（完全初期化。メンテ再起動では使わない）curl -X POST http://127.0.0.1:3000/admin/wipe");
    println!("  GET /ws          -> WebSocket（状態配信・行動受付）");
    println!("  行動例: 送信 {{\"action\":\"end_turn\"}} でターン進行");
    println!("  遺跡: 30秒ごとにスポーン判定、最大5個");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}
