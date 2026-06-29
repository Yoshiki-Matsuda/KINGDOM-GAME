use crate::{
    app_state::{AppState, GameStore},
    config,
    model::tick_world,
    persistence,
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
                        let _ = persistence::save_state(&state.db_pool, state.pvp_world_id(), "pvp", &game).await;
                    }
                    let game = game.read().await;
                    state.broadcast_json(
                        None,
                        serde_json::to_string(&*game).unwrap_or_default(),
                    );
                }
                GameStore::PerPlayer(_) => {
                    let player_ids = match &state.store {
                        GameStore::PerPlayer(mgr) => mgr.active_player_ids().await,
                        _ => unreachable!(),
                    };
                    let mut handles = Vec::with_capacity(player_ids.len());
                    for player_id in player_ids {
                        let st = state.clone();
                        handles.push(tokio::spawn(async move {
                            let mgr = match &st.store {
                                GameStore::PerPlayer(m) => m,
                                _ => unreachable!(),
                            };
                            let world = mgr.get_or_create_world(&player_id).await;
                            mgr.touch(&player_id).await;
                            {
                                let _guard = st.mutation_lock.lock().await;
                                let mut game = world.write().await;
                                tick_world(&mut game, dev_auto_win, server_mode);
                                let _ = persistence::save_player_world(&st.db_pool, &player_id, &game).await;
                            }
                            let game = world.read().await;
                            mgr.broadcast(
                                &player_id,
                                serde_json::to_string(&*game).unwrap_or_default(),
                            ).await;
                        }));
                    }
                    for h in handles {
                        let _ = h.await;
                    }
                }
            }
        }
    });
}
