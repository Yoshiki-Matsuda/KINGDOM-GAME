use super::*;

pub(crate) fn build_game_state(
    state: &GameState,
    turn: u32,
    territories: Vec<Territory>,
    log: Vec<String>,
    players: HashMap<String, PlayerData>,
) -> GameState {
    GameState {
        turn,
        phase: state.phase.clone(),
        territories,
        log,
        players,
        alliances: state.alliances.clone(),
        season: state.season.clone(),
        market_listings: state.market_listings.clone(),
    }
}

pub(crate) const MAX_LOG_LINES: usize = 2000;

impl Default for GameState {
    fn default() -> Self {
        let home_territory_id = format!("c_{}_{}", HOME_COL, HOME_ROW);
        let default_player = PlayerData::new(
            DEFAULT_PLAYER_ID.to_string(),
            home_territory_id,
        );
        let mut players = HashMap::new();
        players.insert(DEFAULT_PLAYER_ID.to_string(), default_player.clone());
        
        Self {
            turn: 1,
            phase: "idle".to_string(),
            territories: default_territories(),
            log: vec![],
            players,
            alliances: vec![],
            season: SeasonInfo::default(),
            market_listings: vec![],
        }
    }
}

/// 開発用: 初期アイテムを追加
pub(super) fn default_dev_inventory() -> Vec<InventoryItem> {
    vec![
        // 基本素材（大量）
        InventoryItem { item_id: "ancient_stone".to_string(), count: 500 },
        InventoryItem { item_id: "rusty_gear".to_string(), count: 200 },
        InventoryItem { item_id: "rotten_wood".to_string(), count: 300 },
        InventoryItem { item_id: "broken_brick".to_string(), count: 200 },
        // 中級素材
        InventoryItem { item_id: "mystic_crystal".to_string(), count: 100 },
        InventoryItem { item_id: "magic_shard".to_string(), count: 150 },
        InventoryItem { item_id: "refined_iron".to_string(), count: 100 },
        InventoryItem { item_id: "reinforced_fiber".to_string(), count: 80 },
        InventoryItem { item_id: "ancient_blueprint".to_string(), count: 30 },
        // 上級素材
        InventoryItem { item_id: "shining_magicstone".to_string(), count: 50 },
        InventoryItem { item_id: "golden_gear".to_string(), count: 20 },
        // レア素材
        InventoryItem { item_id: "guardian_core".to_string(), count: 10 },
        InventoryItem { item_id: "ancient_kings_seal".to_string(), count: 5 },
        InventoryItem { item_id: "dragon_scale".to_string(), count: 3 },
    ]
}

/// 初期所持魔獣（北欧神話キャラ）
pub(super) fn default_owned_cards() -> Vec<u32> {
    // 魔獣マスタID 0〜9: 初期所持（各1枠）
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}

/// 領地レベルから資源生産量を算出（KC準拠: 高レベル領地ほど多く生産）
fn resource_rate_for_level(level: u8) -> (u64, u64, u64, u64) {
    match level {
        1 => (10, 10, 5, 3),
        2 => (15, 15, 8, 5),
        3 => (20, 20, 12, 8),
        4 => (25, 25, 18, 12),
        5 => (30, 30, 25, 18),
        _ => (35, 35, 30, 20),
    }
}

/// 時間ベース資源生産: 占領した領地の数・レベルに応じて資源が増加
pub fn tick_resources(state: &mut GameState) {
    let now = default_now_ms();

    for (player_id, player) in state.players.iter_mut() {
        let player_territories: Vec<u8> = state.territories.iter()
            .filter(|t| t.owner_id.as_deref() == Some(player_id.as_str()))
            .map(|t| t.level)
            .collect();
        let elapsed_ms = now.saturating_sub(player.last_resource_tick);
        if elapsed_ms < 60_000 { continue; }

        let minutes = elapsed_ms / 60_000;
        let (mut food_rate, mut wood_rate, mut stone_rate, mut iron_rate) = (5u64, 5u64, 3u64, 2u64);
        for &level in &player_territories {
            let (f, w, s, i) = resource_rate_for_level(level);
            food_rate += f;
            wood_rate += w;
            stone_rate += s;
            iron_rate += i;
        }

        let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
        let res_cap = 10_000u64.saturating_add(bonuses.storage_capacity as u64 * 150);

        player.resources.food = (player.resources.food + food_rate * minutes).min(res_cap);
        player.resources.wood = (player.resources.wood + wood_rate * minutes).min(res_cap);
        player.resources.stone = (player.resources.stone + stone_rate * minutes).min(res_cap);
        player.resources.iron = (player.resources.iron + iron_rate * minutes).min(res_cap);
        player.last_resource_tick = now;
    }
}

/// 純粋関数: 行動を適用した新状態を返す。戦闘はすべてここで処理し、ログに記録する。
/// `dev_auto_win`: true のとき攻撃側を10倍有利に（戦闘計算・ログは通常表示、ローカル開発用）。
pub fn apply_action(
    state: &GameState,
    actor_player_id: &str,
    action: &Action,
    dev_auto_win: bool,
) -> GameState {
    crate::model_actions::apply_action(state, actor_player_id, action, dev_auto_win)
}

/// 期限切れの遺跡をクリーンアップ
/// 遺跡が期限切れになったら、遺跡を削除して元の中立マスに戻す
pub fn cleanup_expired_ruins(state: &mut GameState) -> bool {
    crate::model_ruins::cleanup_expired_ruins(state)
}

/// 遺跡をランダムな中立マスにスポーンさせる
/// 成功したらtrue、スポーン先がなければfalse
pub fn spawn_random_ruin(state: &mut GameState) -> bool {
    crate::model_ruins::spawn_random_ruin(state)
}

/// 現在の遺跡数をカウント
pub fn count_ruins(state: &GameState) -> usize {
    crate::model_ruins::count_ruins(state)
}

/// シーズン終了チェック: 期間を過ぎたらマップ・領地をリセットして新シーズン開始
pub fn check_season_end(state: &mut GameState) -> bool {
    let now = default_now_ms();
    let elapsed = now.saturating_sub(state.season.started_at);
    if elapsed < state.season.duration_ms {
        return false;
    }

    let old_season = state.season.season_number;
    state.season = SeasonInfo {
        season_number: old_season + 1,
        started_at: now,
        duration_ms: state.season.duration_ms,
    };

    state.territories = default_territories();

    for player in state.players.values_mut() {
        player.resources = Resources::default();
        player.explorations.clear();
    }

    state.alliances.clear();

    push_log(&mut state.log, format!(
        "シーズン{}が終了しました！シーズン{}が開始されます。",
        old_season, old_season + 1
    ));
    true
}
