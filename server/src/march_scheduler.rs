//! 遠征到着の即時処理（ワールド tick とは独立）
//!
//! `arrives_at` を過ぎた攻撃・援軍・探索は専用タスクが即時処理する（帰還は接続時 tick で処理）。

use std::pin::Pin;
use std::time::Duration;

use tokio::time::{sleep, Instant, Sleep};

use crate::{
    app_state::{AppState, GameStore},
    model::{ms_until_march_processing, tick_march_arrivals},
    persistence::{save_player_world, save_state},
};

pub(crate) fn spawn_march_scheduler(state: AppState) {
    tokio::spawn(async move {
        loop {
            let sleep_ms = next_sleep_ms(&state).await;
            wait_until_due_or_wake(&state, sleep_ms).await;
            let _ = process_all_march_arrivals(&state).await;
        }
    });
}

async fn next_sleep_ms(state: &AppState) -> u64 {
    let include_returns = state.server_mode == crate::server_mode::ServerMode::Pvp;
    match &state.store {
        GameStore::Shared(game) => {
            let game = game.read().await;
            ms_until_march_processing(&game, include_returns)
        }
        GameStore::PerPlayer(mgr) => {
            let mut min_wait = crate::config::march_idle_poll_ms();
            let player_ids = mgr.player_ids_for_march_processing().await;
            for player_id in player_ids {
                let world = mgr.get_or_create_world(&player_id).await;
                let game = world.read().await;
                min_wait = min_wait.min(ms_until_march_processing(&game, false));
            }
            min_wait
        }
    }
}

async fn wait_until_due_or_wake(state: &AppState, sleep_ms: u64) {
    if sleep_ms == 0 {
        return;
    }
    let deadline = Instant::now() + Duration::from_millis(sleep_ms);
    let mut sleep_fut: Pin<Box<Sleep>> = Box::pin(sleep_until(deadline));
    tokio::select! {
        _ = sleep_fut.as_mut() => {}
        _ = state.march_wake.notified() => {}
    }
}

fn sleep_until(deadline: Instant) -> Sleep {
    sleep(deadline.saturating_duration_since(Instant::now()))
}

async fn process_all_march_arrivals(state: &AppState) -> bool {
    match &state.store {
        GameStore::Shared(game) => process_shared_world(state, game).await,
        GameStore::PerPlayer(mgr) => {
            let mut any = false;
            let player_ids = mgr.player_ids_for_march_processing().await;
            for player_id in player_ids {
                let world = mgr.get_or_create_world(&player_id).await;
                if process_player_world(state, mgr, &player_id, world).await {
                    mgr.touch(&player_id).await;
                    any = true;
                }
            }
            any
        }
    }
}

async fn process_shared_world(
    state: &AppState,
    game: &std::sync::Arc<tokio::sync::RwLock<crate::model::GameState>>,
) -> bool {
    let changed = {
        let _guard = state.mutation_lock.lock().await;
        let mut game = game.write().await;
        let changed = tick_march_arrivals(&mut game, state.dev_auto_win, state.server_mode);
        if changed {
            let _ = save_state(&state.state_path, &game).await;
        }
        changed
    };
    if changed {
        let game = game.read().await;
        state.broadcast_json(None, serde_json::to_string(&*game).unwrap_or_default());
    }
    changed
}

async fn process_player_world(
    state: &AppState,
    mgr: &std::sync::Arc<crate::world_manager::WorldManager>,
    player_id: &str,
    world: std::sync::Arc<tokio::sync::RwLock<crate::model::GameState>>,
) -> bool {
    let changed = {
        let _guard = state.mutation_lock.lock().await;
        let mut game = world.write().await;
        let changed = tick_march_arrivals(&mut game, state.dev_auto_win, state.server_mode);
        if changed {
            let _ = save_player_world(mgr.base_path(), player_id, &game).await;
        }
        changed
    };
    if changed {
        let game = world.read().await;
        mgr.broadcast(player_id, serde_json::to_string(&*game).unwrap_or_default())
            .await;
    }
    changed
}
