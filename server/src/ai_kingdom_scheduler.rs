use crate::{
    ai_actions::run_ai_faction_turn,
    app_state::{AppState, GameStore},
    config,
    model::{push_system_event, AiPersonality},
};

fn personality_label(personality: AiPersonality) -> &'static str {
    match personality {
        AiPersonality::Aggressive => "aggressive",
        AiPersonality::Balanced => "balanced",
        AiPersonality::Defensive => "defensive",
    }
}

fn faction_display_name(state: &crate::model::GameState, ai_id: &str) -> String {
    state
        .ai_factions
        .iter()
        .find(|f| format!("ai_{}", f.faction_id) == ai_id)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| ai_id.to_string())
}

pub(crate) fn spawn_ai_kingdom_scheduler(state: AppState) {
    tokio::spawn(async move {
        let interval_secs = config::ai_tick_interval_secs();

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

            let GameStore::PerPlayer(mgr) = &state.store else {
                continue;
            };

            let player_ids = mgr.active_player_ids().await;
            if player_ids.is_empty() {
                continue;
            }

            for player_id in player_ids {
                let world = mgr.get_or_create_world(&player_id).await;
                mgr.touch(&player_id).await;
                let _guard = state.mutation_lock.lock().await;
                let snapshot = world.read().await.clone();
                let owner_id = snapshot
                    .world_owner_id
                    .clone()
                    .unwrap_or_else(|| player_id.clone());
                let factions: Vec<(String, AiPersonality)> = snapshot
                    .ai_factions
                    .iter()
                    .map(|f| (format!("ai_{}", f.faction_id), f.personality))
                    .collect();

                println!(
                    "[kingdom-server] AI tick 開始 world={player_id} 勢力={} interval={interval_secs}s",
                    factions.len()
                );

                let mut current = snapshot;
                for (ai_id, personality) in factions {
                    if !current.players.contains_key(&ai_id) {
                        continue;
                    }
                    let faction_name = faction_display_name(&current, &ai_id);
                    let home_id = current
                        .players
                        .get(&ai_id)
                        .map(|p| p.home_territory_id.clone());
                    let home_alive = home_id
                        .as_deref()
                        .and_then(|hid| {
                            current
                                .territories
                                .iter()
                                .find(|t| t.id == hid)
                                .and_then(|t| t.owner_id.clone())
                        })
                        .map(|o| o == ai_id)
                        .unwrap_or(false);
                    if !home_alive {
                        current.ai_factions.retain(|f| format!("ai_{}", f.faction_id) != ai_id);
                        current.players.remove(&ai_id);
                        push_system_event(
                            &mut current.log,
                            &format!("【AI勢力】{ai_id} が滅亡しました。"),
                        );
                        println!(
                            "[kingdom-server] AI tick {ai_id} ({faction_name}) 滅亡 home={}",
                            home_id.unwrap_or_default()
                        );
                        continue;
                    }
                    let (next_state, report) = run_ai_faction_turn(
                        &current,
                        &ai_id,
                        personality,
                        &owner_id,
                        state.dev_auto_win,
                    );
                    current = next_state;
                    println!(
                        "[kingdom-server] AI tick {ai_id} ({faction_name}, {}): {}",
                        personality_label(personality),
                        report.summarize()
                    );
                }

                {
                    let mut game = world.write().await;
                    *game = current.clone();
                }
                mgr.save_world(&player_id, &current).await;
                let json = serde_json::to_string(&current).unwrap_or_default();
                mgr.broadcast(&player_id, json).await;
                state.wake_march_scheduler_if_active(&current);

                println!("[kingdom-server] AI tick 完了 world={player_id}");
            }
        }
    });
}
