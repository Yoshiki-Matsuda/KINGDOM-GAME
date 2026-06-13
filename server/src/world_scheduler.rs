use crate::{
    app_state::{AppState, GameStore},
    config,
    model::tick_world,
    persistence::{save_player_world, save_state},
};

pub(crate) fn spawn_world_scheduler(state: AppState) {
    tokio::spawn(async move {
        let interval_secs = config::world_tick_secs();
        let dev_auto_win = state.dev_auto_win;
        let server_mode = state.server_mode;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

            match &state.store {
                GameStore::Shared(game) => {
                    {
                        let _guard = state.mutation_lock.lock().await;
                        let mut game = game.write().await;
                        tick_world(&mut game, dev_auto_win, server_mode);
                        let _ = save_state(&state.state_path, &game).await;
                    }
                    let game = game.read().await;
                    state.broadcast_json(
                        None,
                        serde_json::to_string(&*game).unwrap_or_default(),
                    );
                }
                GameStore::PerPlayer(mgr) => {
                    let player_ids = mgr.active_player_ids().await;
                    for player_id in player_ids {
                        let world = mgr.get_or_create_world(&player_id).await;
                        mgr.touch(&player_id).await;
                        {
                            let _guard = state.mutation_lock.lock().await;
                            let mut game = world.write().await;
                            tick_world(&mut game, dev_auto_win, server_mode);
                            let _ =
                                save_player_world(mgr.base_path(), &player_id, &game).await;
                        }
                        let game = world.read().await;
                        mgr.broadcast(
                            &player_id,
                            serde_json::to_string(&*game).unwrap_or_default(),
                        )
                        .await;
                    }
                }
            }
        }
    });
}
