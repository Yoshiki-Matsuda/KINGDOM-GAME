use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use tokio::sync::broadcast::error::RecvError;

use crate::{
    app_state::{AppState, GameStore},
    auth,
    model::{
        apply_action, check_season_end, client_view_json, tick_world,
        Action, GameState,
    },
    persistence,
    server_mode::ServerMode,
};

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum ClientEnvelope {
    #[serde(rename = "auth")]
    Auth { token: String },
}

pub(crate) async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let Some(actor_player_id) = authenticate_socket(&mut socket, &state).await else {
        return;
    };

    let server_mode = state.server_mode;
    let is_pvp = server_mode == ServerMode::Pvp;

    let (world_arc, mut broadcast_rx) = match &state.store {
        GameStore::Shared(game) => {
            let json = {
                let _guard = state.mutation_lock.lock().await;
                let mut game = game.write().await;
                tick_world(&mut game, state.dev_auto_win, server_mode);
                if is_pvp {
                    check_season_end(&mut game);
                }
                state.wake_march_scheduler_if_active(&game);
                let _ = persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &game).await;
                if is_pvp {
                    client_view_json(&game, &actor_player_id, server_mode)
                } else {
                    serde_json::to_string(&*game)
                        .unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
                }
            };
            if socket.send(Message::Text(json)).await.is_err() {
                return;
            }
            (game.clone(), state.broadcast_tx.subscribe())
        }
        GameStore::PerPlayer(mgr) => {
            let mgr = mgr.clone();
            let world = mgr.get_or_create_world(&actor_player_id).await;
            mgr.touch(&actor_player_id).await;
            let json = {
                let _guard = state.mutation_lock.lock().await;
                let mut game = world.write().await;
                tick_world(&mut game, state.dev_auto_win, server_mode);
                state.wake_march_scheduler_if_active(&game);
                let _ = persistence::save_player_world(&state.db_pool, &actor_player_id, &game).await;
                serde_json::to_string(&*game)
                    .unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
            };
            if socket.send(Message::Text(json)).await.is_err() {
                return;
            };
            let rx = mgr
                .subscribe(&actor_player_id)
                .await
                .unwrap_or_else(|| {
                    let (_, rx) = tokio::sync::broadcast::channel(1);
                    rx
                });
            (world, rx)
        }
    };

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
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                let Ok(action) = serde_json::from_str::<Action>(text) else { continue };

                if let GameStore::PerPlayer(mgr) = &state.store {
                    mgr.touch(&actor_player_id).await;
                }

                let _guard = state.mutation_lock.lock().await;
                let game_snapshot = { world_arc.read().await.clone() };
                let dev_auto_win = state.dev_auto_win;
                let actor_for_apply = actor_player_id.clone();
                let action_for_log = action.clone();
                let server_mode = state.server_mode;
                let new_state = match tokio::task::spawn_blocking(move || {
                    let mut updated = apply_action(
                        &game_snapshot,
                        &actor_for_apply,
                        &action,
                        dev_auto_win,
                        server_mode,
                    );
                    tick_world(&mut updated, dev_auto_win, server_mode);
                    if server_mode == ServerMode::Pvp {
                        check_season_end(&mut updated);
                    }
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
                        .map(|territory| territory.owner_id.as_deref() == Some(actor_player_id.as_str()))
                        .unwrap_or(false);
                    println!(
                        "[kingdom-server] 攻撃処理: to={} conquered={}",
                        to_territory_id,
                        conquered
                    );
                }

                {
                    let mut game = world_arc.write().await;
                    *game = new_state.clone();
                }

                persist_and_broadcast(&state, &actor_player_id, &new_state).await;

                let json = if is_pvp {
                    client_view_json(&new_state, &actor_player_id, server_mode)
                } else {
                    serde_json::to_string(&new_state).unwrap_or_else(|e| {
                        eprintln!("[kingdom-server] GameState serialize error: {}", e);
                        r#"{"error":"serialize"}"#.to_string()
                    })
                };
                let _ = socket.send(Message::Text(json)).await;
            }
            br = broadcast_rx.recv() => {
                match br {
                    Ok(full_json) => {
                        let json = if is_pvp {
                            filter_broadcast_json(&full_json, &actor_player_id, server_mode)
                        } else {
                            full_json
                        };
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => break,
                }
            }
        }
    }
}

fn filter_broadcast_json(full_json: &str, viewer_id: &str, mode: ServerMode) -> String {
    match serde_json::from_str::<GameState>(full_json) {
        Ok(state) => client_view_json(&state, viewer_id, mode),
        Err(_) => full_json.to_string(),
    }
}

async fn persist_and_broadcast(state: &AppState, player_id: &str, new_state: &GameState) {
    let json = serde_json::to_string(new_state).unwrap_or_default();
    match &state.store {
        GameStore::Shared(_) => {
            let _ = persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", new_state).await;
            state.broadcast_json(None, json);
        }
        GameStore::PerPlayer(mgr) => {
            mgr.save_world(player_id, new_state).await;
            mgr.broadcast(player_id, json.clone()).await;
        }
    }
    state.wake_march_scheduler_if_active(new_state);
}

async fn authenticate_socket(socket: &mut WebSocket, state: &AppState) -> Option<String> {
    let msg = match socket.recv().await {
        Some(Ok(msg)) => msg,
        _ => return None,
    };
    let Ok(text) = msg.to_text() else {
        let _ = socket
            .send(Message::Text(r#"{"error":"auth_required"}"#.to_string()))
            .await;
        return None;
    };
    let Ok(ClientEnvelope::Auth { token }) = serde_json::from_str::<ClientEnvelope>(text.trim()) else {
        let _ = socket
            .send(Message::Text(r#"{"error":"auth_required"}"#.to_string()))
            .await;
        return None;
    };
    match auth::verify_token_for_mode(&state.jwt_secret, &token, state.server_mode) {
        Ok(claims) => Some(claims.sub),
        Err(_) => {
            let _ = socket
                .send(Message::Text(r#"{"error":"auth_invalid"}"#.to_string()))
                .await;
            None
        }
    }
}
