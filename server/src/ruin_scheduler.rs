use crate::{
    app_state::AppState,
    model::{cleanup_expired_ruins, count_ruins, spawn_random_ruin},
    persistence::save_state,
};

const MAX_RUINS: usize = 3;
const SPAWN_INTERVAL_SECS: u64 = 60;
const SPAWN_CHANCE: f64 = 0.30;

pub(crate) fn spawn_ruin_scheduler(state: AppState) {
    tokio::spawn(async move {
        use rand::Rng;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(SPAWN_INTERVAL_SECS)).await;

            let mut changed = false;
            {
                let mut game = state.game.write().await;

                if cleanup_expired_ruins(&mut game) {
                    changed = true;
                    println!("[kingdom-server] 期限切れの遺跡をクリーンアップしました");
                }

                let current_count = count_ruins(&game);
                if current_count < MAX_RUINS {
                    let roll: f64 = rand::thread_rng().gen();
                    if roll < SPAWN_CHANCE && spawn_random_ruin(&mut game) {
                        changed = true;
                        println!(
                            "[kingdom-server] 新しい遺跡が出現しました！ (現在: {}個)",
                            current_count + 1
                        );
                    }
                }
            }

            if changed {
                let game = state.game.read().await;
                let _ = save_state(&state.state_path, &game).await;
                let json = serde_json::to_string(&*game).unwrap_or_default();
                let _ = state.broadcast_tx.send(json);
            }
        }
    });
}
