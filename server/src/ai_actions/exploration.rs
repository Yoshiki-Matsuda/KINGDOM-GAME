use super::*;

pub(crate) fn active_explore_march_count(state: &GameState, ai_id: &str) -> usize {
    state
        .players
        .get(ai_id)
        .map(|p| {
            p.marches
                .iter()
                .filter(|m| m.kind == MarchKind::Explore)
                .count()
        })
        .unwrap_or(0)
}

/// 探索の出発領地（隣接する自領。本拠が隣なら本拠から）
pub(crate) fn select_explore_from_territory(
    state: &GameState,
    ai_id: &str,
    to_id: &str,
) -> Option<String> {
    let home_id = state.players.get(ai_id)?.home_territory_id.clone();
    if territories_are_adjacent(&home_id, to_id) {
        return Some(home_id);
    }
    state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(ai_id) && t.id != to_id)
        .filter(|t| territories_are_adjacent(&t.id, to_id))
        .max_by_key(|t| t.troops)
        .map(|t| t.id.clone())
}

pub(crate) fn pick_exploration_territory(state: &GameState, ai_id: &str) -> Option<String> {
    let player = state.players.get(ai_id)?;
    let home_id = &player.home_territory_id;
    let in_flight: HashSet<String> = player
        .marches
        .iter()
        .filter(|m| m.kind == MarchKind::Explore)
        .map(|m| m.to_territory_id.clone())
        .collect();

    let mut candidates: Vec<&Territory> = state
        .territories
        .iter()
        .filter(|t| {
            t.owner_id.as_deref() == Some(ai_id)
                && !t.is_base
                && t.ruin.is_none()
                && t.id != *home_id
                && !in_flight.contains(&t.id)
                && select_explore_from_territory(state, ai_id, &t.id).is_some()
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }

    let low_resources = player.resources.food < 1200
        || player.resources.wood < 800
        || player.resources.stone < 800
        || player.resources.iron < 500;
    candidates.sort_by(|a, b| {
        let level_cmp = if low_resources {
            a.level.cmp(&b.level)
        } else {
            b.level.cmp(&a.level)
        };
        level_cmp.then_with(|| rand::random::<u8>().cmp(&rand::random()))
    });
    candidates.first().map(|t| t.id.clone())
}

pub(crate) fn run_exploration_dispatches(
    state: &mut GameState,
    ai_id: &str,
    dev_auto_win: bool,
    report: &mut AiTurnReport,
) {
    loop {
        let max_slots = state
            .players
            .get(ai_id)
            .map(|p| exploration_max_slots(p.exploration_level))
            .unwrap_or(1);
        let before_len = active_explore_march_count(state, ai_id);
        if before_len >= max_slots {
            break;
        }
        let Some(target_id) = pick_exploration_territory(state, ai_id) else {
            break;
        };
        let Some(from_id) = select_explore_from_territory(state, ai_id, &target_id) else {
            break;
        };
        let Some(card_indices) = pick_exploration_cards(state, ai_id) else {
            break;
        };

        let count = card_indices.len() as u32;
        let monsters_per_body: Vec<u32> = card_indices
            .iter()
            .map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.card_monster_counts.get(i))
                    .copied()
                    .unwrap_or(1)
            })
            .collect();
        let body_names: Vec<String> = card_indices
            .iter()
            .filter_map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.owned_cards.get(i))
                    .copied()
            })
            .filter_map(|cid| get_card(cid).map(|c| c.name.to_string()))
            .collect();
        let speed_per_body: Vec<u32> = card_indices
            .iter()
            .filter_map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.owned_cards.get(i))
                    .copied()
            })
            .map(|cid| get_card(cid).map(|c| c.stats.speed).unwrap_or(5))
            .collect();
        let next = apply_action(
            state,
            ai_id,
            &Action::StartMarch {
                kind: MarchKind::Explore,
                from_territory_id: from_id,
                to_territory_id: target_id.clone(),
                count,
                monsters_per_body: Some(monsters_per_body),
                body_names: Some(body_names),
                unit_name: Some(format!("{ai_id}探索")),
                speed_per_body: Some(speed_per_body),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(card_indices),
                formed_unit_id: None,
            },
            dev_auto_win,
            ServerMode::Pve,
        );
        let after_len = active_explore_march_count(&next, ai_id);
        if after_len <= before_len {
            break;
        }
        if report.exploration_started.is_none() {
            report.exploration_started = Some(target_id);
        }
        *state = next;
    }
}

pub(crate) fn pick_exploration_cards(state: &GameState, ai_id: &str) -> Option<Vec<usize>> {
    let player = state.players.get(ai_id)?;
    let now = crate::model::default_now_ms();
    let locked = march_locked_card_slots(player, now);
    for (i, st) in player.card_stamina.iter().enumerate() {
        if locked.contains(&i) {
            continue;
        }
        if *st >= STAMINA_EXPLORE {
            if i < player.card_rest_until.len() && player.card_rest_until[i] > now {
                continue;
            }
            return Some(vec![i]);
        }
    }
    None
}
