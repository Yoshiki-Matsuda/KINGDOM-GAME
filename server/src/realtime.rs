use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};

use crate::{
    app_state::AppState,
    model::{apply_action, cleanup_expired_ruins, Action},
    persistence::save_state,
};

pub(crate) async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let json = {
        let mut game = state.game.write().await;
        if cleanup_expired_ruins(&mut game) {
            let _ = save_state(&state.state_path, &game).await;
        }
        serde_json::to_string(&*game).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
    };
    if socket.send(Message::Text(json)).await.is_err() {
        return;
    }

    let mut broadcast_rx = state.broadcast_tx.subscribe();

    loop {
        tokio::select! {
            Some(Ok(message)) = socket.recv() => {
                let Ok(text) = message.to_text() else { continue };
                let Ok(action) = serde_json::from_str::<Action>(text) else { continue };
                let new_state = {
                    let game = state.game.read().await;
                    let updated = apply_action(&game, &action, state.dev_auto_win);
                    if let Action::Attack { to_territory_id, .. } = &action {
                        let conquered = updated
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
                    updated
                };
                {
                    let mut game = state.game.write().await;
                    *game = new_state.clone();
                }
                let _ = save_state(&state.state_path, &new_state).await;
                let json = serde_json::to_string(&new_state).unwrap_or_default();
                let _ = state.broadcast_tx.send(json.clone());
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
