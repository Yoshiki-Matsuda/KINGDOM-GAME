use super::*;

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
    player.card_stamina = vec![120; n];
    player.card_levels = vec![1; n];
    player.card_exp = vec![0; n];
    player.card_status_points = vec![0; n];
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

pub(crate) fn ensure_player_in_game(state: &mut GameState, player_id: &str) {
    if state.players.contains_key(player_id) {
        if is_test_player(player_id) {
            refresh_all_test_players(state);
        }
        return;
    }

    let home_territory_id = allocate_home_territory(&mut state.territories, player_id)
        .unwrap_or_else(home_territory_id);
    let mut player = PlayerData::new(player_id.to_string(), home_territory_id);
    if is_test_player(player_id) {
        bootstrap_test_player(&mut player);
    }
    sync_home_territory_body_counts_from_player(&mut state.territories, &player);
    state.players.insert(player_id.to_string(), player);
}

fn allocate_home_territory(territories: &mut [Territory], player_id: &str) -> Option<String> {
    let territory = territories
        .iter_mut()
        .find(|territory| territory.owner_id.is_none() && territory.ruin.is_none())?;
    territory.owner_id = Some(player_id.to_string());
    territory.is_base = true;
    territory.durability = 0;
    territory.max_durability = 0;
    territory.tower_level = 0;
    territory.body_names = None;
    Some(territory.id.clone())
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

pub(crate) fn push_log(log: &mut Vec<String>, line: String) {
    let ts = default_now_ms();
    log.push(format!("[ts:{}]{}", ts, line));
    if log.len() > MAX_LOG_LINES {
        log.drain(0..log.len() - MAX_LOG_LINES);
    }
}

/// 旧ログ（[ts:] プレフィックスなし）にタイムスタンプを付与するマイグレーション。
/// 既存行は起動時刻から逆算して等間隔に並べる。
pub fn migrate_log_timestamps(state: &mut GameState) {
    let now = default_now_ms();
    let total = state.log.len() as u64;
    if total == 0 { return; }
    let mut migrated = false;
    for (i, line) in state.log.iter_mut().enumerate() {
        if line.starts_with("[ts:") { continue; }
        let synthetic_ts = now.saturating_sub((total - i as u64) * 500);
        *line = format!("[ts:{}]{}", synthetic_ts, line);
        migrated = true;
    }
    if migrated {
        println!("[kingdom-server] 旧ログ {} 件にタイムスタンプを付与しました", total);
    }
}
