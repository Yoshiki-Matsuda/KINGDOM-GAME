use super::*;

use std::collections::HashSet;

/// KC: 編成3枠のうちリーダーはインデックス2（前0・中1・リーダー2）
pub(crate) const KC_LEADER_FORMATION_SLOT_INDEX: usize = 2;

/// リーダー枠が埋まっていれば出撃・守備の対象にできる
pub(crate) fn is_kc_unit_ready_to_deploy(indices: &[i32; 3]) -> bool {
    indices[KC_LEADER_FORMATION_SLOT_INDEX] >= 0
}

/// 前衛→中衛→リーダーの順で、埋まっている本拠スロットを配列化
pub(crate) fn formation_owned_slots_in_slot_order(indices: &[i32; 3]) -> Vec<usize> {
    indices
        .iter()
        .filter_map(|&i| if i >= 0 { Some(i as usize) } else { None })
        .collect()
}

/// 進行中遠征（未到着）で使用中の魔獣スロット（帰還中含む）
pub(crate) fn march_locked_card_slots(player: &PlayerData, now: u64) -> HashSet<usize> {
    let mut locked = HashSet::new();
    for m in &player.marches {
        if m.arrives_at <= now {
            continue;
        }
        if let Some(ref oci) = m.owned_card_indices {
            for &i in oci {
                locked.insert(i);
            }
        }
    }
    locked
}

/// 進行中遠征で使用中の編成ユニットID（帰還中含む）
pub(crate) fn march_busy_formed_unit_ids(player: &PlayerData, now: u64) -> HashSet<String> {
    player
        .marches
        .iter()
        .filter(|m| m.arrives_at > now)
        .filter_map(|m| m.formed_unit_id.as_ref())
        .cloned()
        .collect()
}

/// 開発・自動ログイン用。常に最大編成＋魔獣数MAXを維持する。
pub(crate) const TEST_PLAYER_IDS: &[&str] = &["offline_test", "player"];

pub(crate) fn is_test_player(player_id: &str) -> bool {
    TEST_PLAYER_IDS.contains(&player_id)
}

/// テストアカウント向け: 初期魔獣10体を所持し、各スロット魔獣数を上限まで満タンにする。
pub(crate) fn bootstrap_test_player(player: &mut PlayerData) {
    player.owned_cards = default_owned_cards();
    let n = player.owned_cards.len();
    player.card_monster_counts = vec![MAX_MONSTER_COUNT_PER_CARD_SLOT; n];
    player.card_stamina = vec![crate::config::max_card_stamina(); n];
    player.card_levels = vec![1; n];
    player.card_exp = vec![0; n];
    player.card_status_points = vec![0; n];
    player.card_stat_bonuses = vec![CardStatBonuses::default(); n];
    player.card_rest_until = vec![0; n];
    player.card_awakened = vec![false; n];
    player.card_enhanced = vec![false; n];
    // 初期魔獣は各1.5コスト×3体=4.5 のため、テスト用に上限を緩める
    player.unit_cost_cap = 10.0;
}

/// 状態内の全テストプレイヤーに bootstrap を適用し、本拠の編成体数・魔獣数を同期する。
pub fn refresh_all_test_players(state: &mut GameState) {
    let ids: Vec<String> = state
        .players
        .keys()
        .filter(|id| is_test_player(id.as_str()))
        .cloned()
        .collect();
    for id in ids {
        if let Some(player) = state.players.get_mut(&id) {
            bootstrap_test_player(player);
        }
        if let Some(player) = state.players.get(&id) {
            sync_home_territory_body_counts_from_player(&mut state.territories, player);
        }
    }
}

pub(crate) fn default_monster_count_for_card_id(card_id: u32) -> u32 {
    get_card(card_id)
        .map(|c| c.stats.monster_count)
        .filter(|&m| m > 0)
        .unwrap_or(1)
        .clamp(MIN_MONSTER_COUNT_PER_CARD_SLOT, MAX_MONSTER_COUNT_PER_CARD_SLOT)
}

pub(crate) fn initial_card_monster_counts_for_owned(owned: &[u32]) -> Vec<u32> {
    owned
        .iter()
        .copied()
        .map(default_monster_count_for_card_id)
        .collect()
}

pub(crate) fn ensure_card_monster_counts(player: &mut PlayerData) {
    let n = player.owned_cards.len();
    if player.card_monster_counts.len() > n {
        player.card_monster_counts.truncate(n);
    }
    while player.card_monster_counts.len() < n {
        let idx = player.card_monster_counts.len();
        let id = player.owned_cards[idx];
        player
            .card_monster_counts
            .push(default_monster_count_for_card_id(id));
    }
    for c in &mut player.card_monster_counts {
        *c = (*c).clamp(MIN_MONSTER_COUNT_PER_CARD_SLOT, MAX_MONSTER_COUNT_PER_CARD_SLOT);
    }
}

/// 本拠領地の `troops` / `body_monster_counts` をプレイヤーの所持魔獣列と一致させる
pub(crate) fn sync_home_territory_body_counts_from_player(
    territories: &mut [Territory],
    player: &PlayerData,
) {
    let home_id = player.home_territory_id.as_str();
    let Some(tidx) = get_territory_index(territories, home_id) else {
        return;
    };
    let n = player.owned_cards.len() as u32;
    territories[tidx].troops = n;
    territories[tidx].body_monster_counts = Some(player.card_monster_counts.clone());
}

pub(crate) const NO_HOME_AVAILABLE_MSG: &str = "配置可能な本拠地がありません。マップが満杯です。";

pub(crate) fn ensure_player_in_game(state: &mut GameState, player_id: &str) -> Result<(), String> {
    if state.players.contains_key(player_id) {
        if is_test_player(player_id) {
            refresh_all_test_players(state);
        }
        return Ok(());
    }

    let home_territory_id = allocate_home_territory(
        &mut state.territories,
        player_id,
        &state.world,
    )
    .ok_or_else(|| NO_HOME_AVAILABLE_MSG.to_string())?;
    let mut player = PlayerData::new(player_id.to_string(), home_territory_id);
    if is_test_player(player_id) {
        bootstrap_test_player(&mut player);
    }
    sync_home_territory_body_counts_from_player(&mut state.territories, &player);
    state.players.insert(player_id.to_string(), player);
    Ok(())
}

fn manhattan_distance(a_id: &str, b_id: &str) -> Option<u16> {
    let (ac, ar) = parse_territory_id(a_id)?;
    let (bc, br) = parse_territory_id(b_id)?;
    Some(
        (ac as i16 - bc as i16).unsigned_abs() as u16
            + (ar as i16 - br as i16).unsigned_abs() as u16,
    )
}

fn min_distance_to_homes(territory_id: &str, existing_homes: &[String]) -> u16 {
    existing_homes
        .iter()
        .filter_map(|home| manhattan_distance(territory_id, home))
        .min()
        .unwrap_or(u16::MAX)
}

fn allocate_home_territory(
    territories: &mut [Territory],
    player_id: &str,
    world: &WorldConfig,
) -> Option<String> {
    let existing_homes: Vec<String> = territories
        .iter()
        .filter(|t| t.is_base && t.owner_id.is_some())
        .map(|t| t.id.clone())
        .collect();

    let pool: Vec<String> = territories
        .iter()
        .filter(|t| t.owner_id.is_none() && t.ruin.is_none())
        .map(|t| t.id.clone())
        .filter(|id| {
            min_distance_to_homes(id, &existing_homes) >= MIN_HOME_SEPARATION as u16
                && can_place_home_with_safe_zone(territories, id, player_id, world)
        })
        .collect();

    let chosen_id = pool
        .into_iter()
        .max_by_key(|id| min_distance_to_homes(id, &existing_homes))?;

    let idx = get_territory_index(territories, &chosen_id)?;
    let territory = &mut territories[idx];
    territory.owner_id = Some(player_id.to_string());
    territory.is_base = true;
    territory.durability = 0;
    territory.max_durability = 0;
    territory.tower_level = 0;
    territory.body_names = None;

    let (col, row) = parse_territory_id(&chosen_id)?;
    apply_home_safe_zone_levels(territories, col as u16, row as u16, world);

    Some(chosen_id)
}

/// この領地に援軍を送れるか（自領・クランメンバー・配下プレイヤーの領）。
pub(crate) fn can_receive_reinforcement(
    territories: &[Territory],
    actor_player_id: &str,
    allied_owner_ids: &[String],
    territory_id: &str,
) -> bool {
    let Some(idx) = get_territory_index(territories, territory_id) else {
        return false;
    };
    let owner = match &territories[idx].owner_id {
        Some(id) => id.as_str(),
        None => return false,
    };
    owner == actor_player_id || allied_owner_ids.iter().any(|id| id.as_str() == owner)
}

pub(crate) fn territory_name<'a>(territories: &'a [Territory], id: &'a str) -> &'a str {
    territories.iter().find(|t| t.id.as_str() == id).map(|t| t.name.as_str()).unwrap_or(id)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_player_rejects_when_no_home_available() {
        let world = WorldConfig {
            cols: 6,
            rows: 6,
            home_col: 3,
            home_row: 3,
            terrain_seed: 1,
        };
        let mut state = GameState {
            world,
            ..GameState::default()
        };
        state.territories = generate_territories(&world, "first", None);
        state.players.insert(
            "first".to_string(),
            PlayerData::new("first".to_string(), format!("c_{}_{}", world.home_col, world.home_row)),
        );

        for territory in state.territories.iter_mut() {
            if territory.owner_id.is_none() {
                territory.owner_id = Some("blocker".to_string());
            }
        }

        let before_count = state.players.len();
        let first_home_owner = state
            .territories
            .iter()
            .find(|t| t.id == format!("c_{}_{}", world.home_col, world.home_row))
            .and_then(|t| t.owner_id.clone());

        let result = ensure_player_in_game(&mut state, "second");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NO_HOME_AVAILABLE_MSG);
        assert_eq!(state.players.len(), before_count);
        assert!(!state.players.contains_key("second"));
        assert_eq!(
            state
                .territories
                .iter()
                .find(|t| t.id == format!("c_{}_{}", world.home_col, world.home_row))
                .and_then(|t| t.owner_id.clone()),
            first_home_owner
        );
    }
}
