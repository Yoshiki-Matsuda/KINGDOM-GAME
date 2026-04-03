use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use tokio::sync::broadcast::error::RecvError;

use crate::{
    app_state::AppState,
    model::{apply_action, check_season_end, cleanup_expired_ruins, tick_resources, Action},
    persistence::save_state,
};

pub(crate) async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let json = {
        let mut game = state.game.write().await;
        cleanup_expired_ruins(&mut game);
        tick_resources(&mut game);
        check_season_end(&mut game);
        let _ = save_state(&state.state_path, &game).await;
        serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
    };
    if socket.send(Message::Text(json)).await.is_err() {
        return;
    }

    let mut broadcast_rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let msg = match incoming {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => {
                        eprintln!("[kingdom-server] WebSocket recv error: {}", e);
                        break;
                    }
                    None => break,
                };
                let Ok(text) = msg.to_text() else { continue };
                // 空フレームは無視（クライアント・プロキシが送ることがあり、EOF パースエラーでログが埋まる）
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                let Ok(action) = serde_json::from_str::<Action>(text) else { continue };

                // 戦闘など重い処理で非同期ランタイムが占有され、ping や broadcast 受信が遅れると切断や Lagged が起きるため、
                // スナップショットをスレッドプールで処理する。
                let game_snapshot = {
                    let g = state.game.read().await;
                    g.clone()
                };
                let dev_auto_win = state.dev_auto_win;
                let action_for_log = action.clone();
                let new_state = match tokio::task::spawn_blocking(move || {
                    let mut updated = apply_action(&game_snapshot, &action, dev_auto_win);
                    tick_resources(&mut updated);
                    check_season_end(&mut updated);
                    updated
                })
                .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[kingdom-server] apply_action task failed: {}", e);
                        continue;
                    }
                };

                if let Action::Attack { to_territory_id, .. } = &action_for_log {
                    let conquered = new_state
                        .territories
                        .iter()
                        .find(|territory| territory.id == *to_territory_id)
                        .map(|territory| territory.owner_id.as_deref() == Some("player"))
                        .unwrap_or(false);
                    println!(
                        "[kingdom-server] 攻撃処理: to={} conquered={}",
                        to_territory_id,
                        conquered
                    );
                }

                {
                    let mut game = state.game.write().await;
                    *game = new_state.clone();
                }
                let _ = save_state(&state.state_path, &new_state).await;
                let json = serde_json::to_string(&new_state).unwrap_or_else(|e| {
                    eprintln!("[kingdom-server] GameState serialize error: {}", e);
                    r#"{"error":"serialize"}"#.to_string()
                });
                let _ = state.broadcast_tx.send(json.clone());
                let _ = socket.send(Message::Text(json)).await;
            }
            br = broadcast_rx.recv() => {
                match br {
                    Ok(json) => {
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => {
                        // 攻撃処理などで受信が遅れただけ。切断せず最新へ追いつく。
                        continue;
                    }
                    Err(RecvError::Closed) => break,
                }
            }
        }
    }
}
