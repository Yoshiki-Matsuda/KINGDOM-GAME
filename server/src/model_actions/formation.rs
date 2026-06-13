use super::*;

use crate::facilities::calculate_facility_bonuses;

pub(super) fn apply_set_formed_units(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    units: &[StoredFormedUnit],
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };

    let bonuses = calculate_facility_bonuses(&player.facilities);
    let max_units = (1 + bonuses.unit_capacity).max(1) as usize;
    if units.len() > max_units {
        push_log(
            log,
            format!("ユニット数が上限（{}）を超えています。", max_units),
        );
        return state.clone();
    }

    let slot_count = player.owned_cards.len();
    for unit in units {
        if unit.id.is_empty() || unit.name.is_empty() {
            push_log(log, "ユニット情報が不正です。".to_string());
            return state.clone();
        }
        for &idx in &unit.indices {
            if idx == -1 {
                continue;
            }
            if idx < 0 || idx as usize >= slot_count {
                push_log(log, "編成スロットが不正です。".to_string());
                return state.clone();
            }
        }
    }

    player.formed_units = units.to_vec();
    build_game_state(state, state.territories.clone(), log.clone(), players)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{GameState, PlayerData, StoredFormedUnit};

    #[test]
    fn set_formed_units_accepts_empty_slot_minus_one() {
        let mut state = GameState::default();
        let player_id = "player".to_string();
        let mut player = PlayerData::new(player_id.clone(), "c_24_24".to_string());
        player.owned_cards = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        state.players.insert(player_id.clone(), player);

        let units = vec![StoredFormedUnit {
            id: "unit-1".to_string(),
            name: "ユニット1".to_string(),
            indices: [2, -1, -1],
        }];
        let mut log = vec![];
        let next = apply_set_formed_units(&state, &mut log, &player_id, &units);

        assert!(log.is_empty());
        assert_eq!(
            next.players.get(&player_id).unwrap().formed_units,
            units
        );
    }

    #[test]
    fn set_formed_units_deserializes_client_json() {
        use crate::model::Action;
        let json = r#"{"action":"set_formed_units","units":[{"id":"unit-1","name":"ユニット1","indices":[0,1,2]}]}"#;
        let action: Action = serde_json::from_str(json).expect("client payload");
        match action {
            Action::SetFormedUnits { units } => {
                assert_eq!(units.len(), 1);
                assert_eq!(units[0].indices, [0, 1, 2]);
            }
            _ => panic!("expected SetFormedUnits"),
        }
    }
}
