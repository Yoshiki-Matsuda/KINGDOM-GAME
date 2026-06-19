use crate::model::{
    build_game_state, ensure_card_stat_bonuses, push_system_event, CardStatBonuses, GameState,
};

pub(super) fn apply_allocate_card_stats(
    state: &GameState,
    log: &mut Vec<crate::model::GameEvent>,
    actor_player_id: &str,
    card_index: usize,
    delta: CardStatBonuses,
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };

    if card_index >= player.owned_cards.len() {
        push_system_event(log, "指定の魔獣スロットが存在しません。");
        return state.clone();
    }

    let spend = delta.total();
    if spend == 0 {
        push_system_event(log, "振り分けるポイントを1以上指定してください。");
        return state.clone();
    }

    while player.card_status_points.len() < player.owned_cards.len() {
        player.card_status_points.push(0);
    }
    ensure_card_stat_bonuses(player);

    let available = player.card_status_points[card_index];
    if spend > available {
        push_system_event(log, &format!(
                "ステータスポイントが足りません（残り {} / 必要 {}）。",
                available, spend
            ));
        return state.clone();
    }

    player.card_status_points[card_index] = available - spend;
    player.card_stat_bonuses[card_index].add_assignments(&delta);

    let name = crate::cards::get_card(player.owned_cards[card_index])
        .map(|c| c.name.to_string())
        .unwrap_or_else(|| format!("魔獣#{}", player.owned_cards[card_index]));
    push_system_event(log, &format!(
            "「{}」にステータスを振り分けました（残りポイント {}）。",
            name, player.card_status_points[card_index]
        ));

    build_game_state(state, state.territories.clone(), log.clone(), players)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{default_owned_cards, PlayerData, DEFAULT_PLAYER_ID};

    #[test]
    fn allocate_card_stats_spends_points_and_applies_bonuses() {
        let mut state = GameState::default();
        let mut player = PlayerData::new(DEFAULT_PLAYER_ID.to_string(), "c_24_24".to_string());
        player.owned_cards = default_owned_cards();
        player.card_status_points = vec![10; player.owned_cards.len()];
        ensure_card_stat_bonuses(&mut player);
        state.players.insert(DEFAULT_PLAYER_ID.to_string(), player);

        let mut log = Vec::new();
        let delta = CardStatBonuses {
            speed: 2,
            attack: 3,
            intelligence: 5,
            defense: 0,
            magic_defense: 0,
        };
        let next = apply_allocate_card_stats(&state, &mut log, DEFAULT_PLAYER_ID, 0, delta);
        let p = &next.players[DEFAULT_PLAYER_ID];
        assert_eq!(p.card_status_points[0], 0);
        assert_eq!(p.card_stat_bonuses[0].attack, 3);
        assert_eq!(p.card_stat_bonuses[0].intelligence, 5);
    }
}
