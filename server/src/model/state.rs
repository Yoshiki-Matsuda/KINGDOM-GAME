use super::*;

const CLIENT_VIEW_LOG_LIMIT: usize = 200;

/// マップ上に表示する進行中の遠征を収集（帰還・到着済みを除く）
pub fn collect_visible_marches(state: &GameState, now_ms: u64) -> Vec<VisibleMarch> {
    let mut out = Vec::new();
    for (owner_id, player) in &state.players {
        for march in &player.marches {
            if march.kind == MarchKind::Return || march.arrives_at <= now_ms {
                continue;
            }
            out.push(VisibleMarch {
                march_id: march.march_id.clone(),
                owner_id: owner_id.clone(),
                kind: march.kind,
                home_territory_id: player.home_territory_id.clone(),
                from_territory_id: Some(march.from_territory_id.clone()),
                to_territory_id: march.to_territory_id.clone(),
                arrives_at: march.arrives_at,
                unit_name: march.unit_name.clone(),
            });
        }
    }
    out
}

/// クライアントへ送る GameState（PVP は閲覧者スコープ、PVE はそのまま）
pub fn client_view_state(state: &GameState, viewer_id: &str, mode: crate::server_mode::ServerMode) -> GameState {
    use crate::server_mode::ServerMode;
    let now_ms = default_now_ms();
    let visible_marches = collect_visible_marches(state, now_ms);

    if mode == ServerMode::Pve {
        let mut view = state.clone();
        view.visible_marches = visible_marches;
        return view;
    }

    let mut players = HashMap::new();
    if let Some(player) = state.players.get(viewer_id) {
        players.insert(viewer_id.to_string(), player.clone());
    }

    let alliances: Vec<Alliance> = state
        .alliances
        .iter()
        .filter(|a| a.member_ids.iter().any(|m| m == viewer_id))
        .cloned()
        .collect();

    let log = if state.log.len() > CLIENT_VIEW_LOG_LIMIT {
        state.log[state.log.len() - CLIENT_VIEW_LOG_LIMIT..].to_vec()
    } else {
        state.log.clone()
    };

    GameState {
        world: state.world,
        world_owner_id: state.world_owner_id.clone(),
        ai_factions: state.ai_factions.clone(),
        territories: state.territories.clone(),
        log,
        players,
        alliances,
        season: state.season.clone(),
        market_listings: state.market_listings.clone(),
        visible_marches,
    }
}

pub fn client_view_json(state: &GameState, viewer_id: &str, mode: crate::server_mode::ServerMode) -> String {
    let view = client_view_state(state, viewer_id, mode);
    serde_json::to_string(&view).unwrap_or_else(|_| r#"{"error":"serialize"}"#.to_string())
}

pub(crate) fn build_game_state(
    state: &GameState,
    territories: Vec<Territory>,
    log: Vec<String>,
    players: HashMap<String, PlayerData>,
) -> GameState {
    GameState {
        world: state.world,
        world_owner_id: state.world_owner_id.clone(),
        ai_factions: state.ai_factions.clone(),
        territories,
        log,
        players,
        alliances: state.alliances.clone(),
        season: state.season.clone(),
        market_listings: state.market_listings.clone(),
        visible_marches: state.visible_marches.clone(),
    }
}

impl Default for GameState {
    fn default() -> Self {
        let mut world = WorldConfig::default();
        world.terrain_seed = crate::model::resolve_terrain_seed(None);
        let home_territory_id = format!("c_{}_{}", world.home_col, world.home_row);
        let default_player = PlayerData::new(
            DEFAULT_PLAYER_ID.to_string(),
            home_territory_id,
        );
        let mut players = HashMap::new();
        players.insert(DEFAULT_PLAYER_ID.to_string(), default_player.clone());
        
        Self {
            world,
            world_owner_id: None,
            ai_factions: vec![],
            territories: generate_territories(&world, DEFAULT_PLAYER_ID, None),
            log: vec![],
            players,
            alliances: vec![],
            season: SeasonInfo::default(),
            market_listings: vec![],
            visible_marches: vec![],
        }
    }
}

/// 開発用: 初期アイテムを追加
pub(crate) fn default_dev_inventory() -> Vec<InventoryItem> {
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
pub(crate) fn default_owned_cards() -> Vec<u32> {
    // 魔獣マスタID 0〜9: 初期所持（各1枠）
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}

/// 施設による資源生産（10分単位・オフライン catch-up）
pub fn tick_facility_resources(state: &mut GameState) {
    let now = default_now_ms();
    let tick_ms = crate::config::facility_resource_tick_ms();

    for player in state.players.values_mut() {
        let elapsed_ms = now.saturating_sub(player.last_resource_tick);
        if elapsed_ms < tick_ms {
            continue;
        }
        let ticks = elapsed_ms / tick_ms;
        let rates = crate::facilities::calculate_facility_resource_rates(&player.facilities);
        let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
        let res_cap = 10_000u64.saturating_add(bonuses.storage_capacity as u64 * 150);

        player.resources.food = (player.resources.food.saturating_add(rates.food_per_tick.saturating_mul(ticks)))
            .min(res_cap);
        player.resources.wood = (player.resources.wood.saturating_add(rates.wood_per_tick.saturating_mul(ticks)))
            .min(res_cap);
        player.resources.stone = (player.resources.stone.saturating_add(rates.stone_per_tick.saturating_mul(ticks)))
            .min(res_cap);
        player.resources.iron = (player.resources.iron.saturating_add(rates.iron_per_tick.saturating_mul(ticks)))
            .min(res_cap);
        player.last_resource_tick = now;
    }
}

/// 時間ベースの魔獣スタミナ回復（KC準拠: 時間経過で各スロット回復）
pub fn tick_stamina(state: &mut GameState, dev_auto_win: bool) {
    let max_stamina = crate::config::max_card_stamina();

    if dev_auto_win {
        for (player_id, player) in state.players.iter_mut() {
            if !crate::pve_world::is_human_player_id(player_id) {
                continue;
            }
            while player.card_stamina.len() < player.owned_cards.len() {
                player.card_stamina.push(max_stamina);
            }
            for st in player.card_stamina.iter_mut() {
                *st = max_stamina;
            }
        }
        return;
    }

    let now = default_now_ms();
    let base_rate = crate::server_mode::stamina_recovery_per_min() as u32;

    for player in state.players.values_mut() {
        let elapsed_ms = now.saturating_sub(player.last_stamina_tick);
        if elapsed_ms < 60_000 {
            continue;
        }
        let minutes = (elapsed_ms / 60_000) as u32;
        let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
        let recovery_per_min = base_rate.saturating_add(bonuses.stamina_recovery_bonus);

        for i in 0..player.card_stamina.len() {
            if i < player.card_rest_until.len() && player.card_rest_until[i] > now {
                continue;
            }
            let current = player.card_stamina[i];
            let recovered = current.saturating_add(recovery_per_min.saturating_mul(minutes));
            player.card_stamina[i] = recovered.min(max_stamina);
        }
        player.last_stamina_tick = now;
    }
}

const MAX_RUINS: usize = 3;
const RUIN_SPAWN_CHANCE: f64 = 0.30;

/// 遺跡の期限切れ処理とランダムスポーン
pub fn tick_ruins(state: &mut GameState) -> bool {
    let mut changed = cleanup_expired_ruins(state);
    let current_count = count_ruins(state);
    if current_count < MAX_RUINS {
        let roll: f64 = rand::Rng::gen(&mut rand::thread_rng());
        if roll < RUIN_SPAWN_CHANCE && spawn_random_ruin(state) {
            changed = true;
        }
    }
    changed
}

/// 統一ワールド tick（施設資源・スタミナ・遺跡・遠征到着）
pub fn tick_world(
    state: &mut GameState,
    dev_auto_win: bool,
    server_mode: crate::server_mode::ServerMode,
) -> bool {
    let mut log = state.log.clone();
    let before_log_len = log.len();

    tick_facility_resources(state);
    tick_stamina(state, dev_auto_win);
    let _ruins_changed = tick_ruins(state);
    let marches_changed = crate::model_actions::tick_marches(
        state,
        &mut log,
        dev_auto_win,
        server_mode,
        true,
    );

    if log.len() != before_log_len || marches_changed {
        state.log = log;
    }
    true
}

/// 遠征到着のみ処理（ワールド tick より高頻度で実行）
/// PVE: 攻撃・援軍・探索のみ（帰還は接続時）。PVP: 帰還含むすべて。
pub fn tick_march_arrivals(
    state: &mut GameState,
    dev_auto_win: bool,
    server_mode: crate::server_mode::ServerMode,
) -> bool {
    let include_returns = server_mode == crate::server_mode::ServerMode::Pvp;
    let mut log = state.log.clone();
    let before_log_len = log.len();
    let marches_changed = crate::model_actions::tick_marches(
        state,
        &mut log,
        dev_auto_win,
        server_mode,
        include_returns,
    );
    if log.len() != before_log_len || marches_changed {
        state.log = log;
    }
    marches_changed || state.log.len() != before_log_len
}

/// 次に遠征到着を処理するまでの待機ミリ秒（到着予定がなければアイドル間隔）
/// `include_returns`: false のとき帰還 March はスケジュール対象外（接続時 tick で処理）
pub fn ms_until_march_processing(state: &GameState, include_returns: bool) -> u64 {
    let now = default_now_ms();
    let march_relevant = |m: &crate::model::MarchMission| {
        include_returns || m.kind != crate::model::MarchKind::Return
    };
    let has_due = state
        .players
        .values()
        .flat_map(|p| &p.marches)
        .any(|m| m.arrives_at <= now && march_relevant(m));
    if has_due {
        return 0;
    }
    let next = state
        .players
        .values()
        .flat_map(|p| &p.marches)
        .filter(|m| march_relevant(m))
        .map(|m| m.arrives_at)
        .filter(|&at| at > now)
        .min();
    match next {
        Some(at) => at.saturating_sub(now).saturating_add(50),
        None => crate::config::march_idle_poll_ms(),
    }
}

/// 純粋関数: 行動を適用した新状態を返す。戦闘はすべてここで処理し、ログに記録する。
/// `dev_auto_win`: true のとき攻撃ダメージ10倍・人間プレイヤーの所持魔獣スタミナ無限（ローカル開発用）。
pub fn apply_action(
    state: &GameState,
    actor_player_id: &str,
    action: &Action,
    dev_auto_win: bool,
    server_mode: crate::server_mode::ServerMode,
) -> GameState {
    crate::model_actions::apply_action(state, actor_player_id, action, dev_auto_win, server_mode)
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

    state.territories = generate_territories(&state.world, DEFAULT_PLAYER_ID, None);

    for player in state.players.values_mut() {
        player.resources = Resources::default();
        player.marches.clear();
    }

    state.alliances.clear();

    push_log(&mut state.log, format!(
        "シーズン{}が終了しました！シーズン{}が開始されます。",
        old_season, old_season + 1
    ));
    true
}

#[cfg(test)]
mod world_tick_tests {
    use super::*;
    use crate::model::{MarchKind, MarchMission, PlayerData, Territory};
    use crate::server_mode::ServerMode;

    fn test_state_with_player(player: PlayerData) -> GameState {
        let mut state = GameState::default();
        state.players.insert(player.player_id.clone(), player);
        state
    }

    #[test]
    fn facility_resource_tick_every_10_minutes() {
        let mut player = PlayerData::new("p".to_string(), "c_24_24".to_string());
        let now = default_now_ms();
        player.last_resource_tick = now.saturating_sub(20 * 60 * 1000);
        player.facilities.push(crate::model::BuiltFacility {
            facility_id: "field".to_string(),
            level: 1,
            build_complete_at: None,
            position: None,
        });
        let mut state = test_state_with_player(player);
        tick_facility_resources(&mut state);
        let p = state.players.get("p").unwrap();
        assert_eq!(p.resources.food, 500 + 5 * 2);
    }

    #[test]
    fn explore_march_arrival_grants_resources() {
        let mut player = PlayerData::new("p".to_string(), "c_24_24".to_string());
        let now = default_now_ms();
        player.marches.push(MarchMission {
            march_id: "m1".to_string(),
            kind: MarchKind::Explore,
            from_territory_id: "c_24_24".to_string(),
            to_territory_id: "c_25_24".to_string(),
            started_at: now.saturating_sub(60_000),
            arrives_at: now.saturating_sub(1),
            count: 1,
            monsters_per_body: Some(vec![10]),
            body_names: Some(vec!["A".to_string()]),
            unit_name: Some("探索隊".to_string()),
            speed_per_body: Some(vec![5]),
            skills_per_body: None,
            stats_per_body: None,
            owned_card_indices: Some(vec![0]),
            formed_unit_id: None,
        });
        let mut state = test_state_with_player(player);
        state.territories.push(Territory {
            id: "c_25_24".to_string(),
            name: "テスト領".to_string(),
            level: 1,
            owner_id: Some("p".to_string()),
            troops: 1,
            body_monster_counts: None,
            body_names: None,
            ruin: None,
            is_base: false,
            durability: 0,
            max_durability: 0,
            tower_level: 0,
        });
        let food_before = state.players.get("p").unwrap().resources.food;
        tick_world(&mut state, false, ServerMode::Pve);
        let p = state.players.get("p").unwrap();
        assert!(p.marches.is_empty());
        assert!(p.resources.food > food_before);
        assert!(state.log.iter().any(|l| l.contains("確実成功")));
    }

    #[test]
    fn exploration_bonus_is_40_percent_of_conquest_range() {
        for _ in 0..30 {
            let (f, w, s, i) = crate::model::exploration_resource_bonus(2);
            // Lv2 占領レンジ 25〜130 の40%（切り捨て）→ 10〜52
            assert!((10..=52).contains(&f));
            assert!((10..=52).contains(&w));
            assert!((10..=52).contains(&s));
            assert!((10..=52).contains(&i));
        }
    }
}

#[cfg(test)]
mod client_view_tests {
    use super::*;

    #[test]
    fn pvp_client_view_hides_other_players() {
        let world = WorldConfig::default();
        let mut state = GameState::default();
        state.world = world;
        state.players.insert(
            "viewer".to_string(),
            PlayerData::new("viewer".to_string(), "c_24_24".to_string()),
        );
        state.players.insert(
            "other".to_string(),
            PlayerData::new("other".to_string(), "c_10_10".to_string()),
        );

        let view = client_view_state(&state, "viewer", crate::server_mode::ServerMode::Pvp);
        assert_eq!(view.players.len(), 1);
        assert!(view.players.contains_key("viewer"));
        assert!(!view.players.contains_key("other"));
    }

    #[test]
    fn pvp_client_view_includes_other_players_visible_marches() {
        use crate::model::{MarchKind, MarchMission};

        let mut state = GameState::default();
        let mut other = PlayerData::new("other".to_string(), "c_10_10".to_string());
        other.marches.push(MarchMission {
            march_id: "m-other-1".to_string(),
            kind: MarchKind::Attack,
            from_territory_id: "c_9_10".to_string(),
            to_territory_id: "c_11_10".to_string(),
            started_at: 0,
            arrives_at: u64::MAX,
            count: 1,
            monsters_per_body: None,
            body_names: None,
            unit_name: Some("敵遠征".to_string()),
            speed_per_body: None,
            skills_per_body: None,
            stats_per_body: None,
            owned_card_indices: None,
            formed_unit_id: None,
        });
        state.players.insert("viewer".to_string(), PlayerData::new("viewer".to_string(), "c_24_24".to_string()));
        state.players.insert("other".to_string(), other);

        let view = client_view_state(&state, "viewer", crate::server_mode::ServerMode::Pvp);
        assert_eq!(view.players.len(), 1);
        assert_eq!(view.visible_marches.len(), 1);
        assert_eq!(view.visible_marches[0].owner_id, "other");
        assert_eq!(view.visible_marches[0].home_territory_id, "c_10_10");
    }

    #[test]
    fn collect_visible_marches_skips_return_and_expired() {
        use crate::model::{MarchKind, MarchMission};

        let mut player = PlayerData::new("p1".to_string(), "c_1_1".to_string());
        player.marches.push(MarchMission {
            march_id: "return".to_string(),
            kind: MarchKind::Return,
            from_territory_id: "c_1_1".to_string(),
            to_territory_id: "c_2_2".to_string(),
            started_at: 0,
            arrives_at: u64::MAX,
            count: 0,
            monsters_per_body: None,
            body_names: None,
            unit_name: None,
            speed_per_body: None,
            skills_per_body: None,
            stats_per_body: None,
            owned_card_indices: None,
            formed_unit_id: None,
        });
        player.marches.push(MarchMission {
            march_id: "attack".to_string(),
            kind: MarchKind::Attack,
            from_territory_id: "c_1_1".to_string(),
            to_territory_id: "c_3_3".to_string(),
            started_at: 0,
            arrives_at: 1000,
            count: 1,
            monsters_per_body: None,
            body_names: None,
            unit_name: None,
            speed_per_body: None,
            skills_per_body: None,
            stats_per_body: None,
            owned_card_indices: None,
            formed_unit_id: None,
        });
        let mut state = GameState::default();
        state.players.insert("p1".to_string(), player);

        let marches = collect_visible_marches(&state, 500);
        assert_eq!(marches.len(), 1);
        assert_eq!(marches[0].march_id, "attack");

        let expired = collect_visible_marches(&state, 1500);
        assert!(expired.is_empty());
    }

    #[test]
    fn pve_client_view_keeps_all_players() {
        let mut state = GameState::default();
        state.world_owner_id = Some("viewer".to_string());
        state.players.clear();
        state.players.insert(
            "viewer".to_string(),
            PlayerData::new("viewer".to_string(), "c_24_24".to_string()),
        );
        state.players.insert(
            "ai_faction_0".to_string(),
            PlayerData::new("ai_faction_0".to_string(), "c_5_5".to_string()),
        );

        let view = client_view_state(&state, "viewer", crate::server_mode::ServerMode::Pve);
        assert_eq!(view.players.len(), 2);
    }
}
