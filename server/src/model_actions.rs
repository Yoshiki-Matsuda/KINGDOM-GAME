use rand::seq::SliceRandom;
use rand::Rng;

use crate::model::{
    attack_base_owner_ids,
    build_game_state,
    can_receive_reinforcement,
    default_now_ms,
    generate_neutral_enemies,
    get_territory_index,
    home_territory_id,
    is_attackable_target,
    territories_are_adjacent,
    is_home_territory,
    parse_territory_coords,
    push_log,
    territory_name,
    wave_count_for_level,
    Action,
    CardStats,
    ExplorationMission,
    GameState,
    MarketItemType,
    MarketListing,
    DEFAULT_PLAYER_ID,
};
use crate::skills::{
    apply_attack_skills,
    apply_battle_start_skills,
    apply_effect_to_character,
    check_death_skills,
    CombatCharacter,
    SkillData,
};

pub(crate) fn apply_action(state: &GameState, action: &Action, dev_auto_win: bool) -> GameState {
    let mut log = state.log.clone();
    let mut result = match action {
        Action::EndTurn => apply_end_turn_action(state, &mut log),
        Action::Deploy {
            territory_id,
            count,
            monsters_per_body,
            body_names,
        } => apply_deploy_action(state, &mut log, territory_id, *count, monsters_per_body, body_names),
        Action::Attack {
            from_territory_id,
            to_territory_id,
            count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
        } => apply_attack_action(
            state,
            &mut log,
            from_territory_id,
            to_territory_id,
            *count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
            dev_auto_win,
        ),
        Action::BuildBase { territory_id } => apply_build_base(state, &mut log, territory_id),
        Action::SynthesizeCard { base_card_index, material_card_indices } => {
            apply_synthesize_card(state, &mut log, *base_card_index, material_card_indices)
        }
        Action::CreateAlliance { name } => apply_create_alliance(state, &mut log, name),
        Action::JoinAlliance { alliance_id } => apply_join_alliance(state, &mut log, alliance_id),
        Action::LeaveAlliance => apply_leave_alliance(state, &mut log),
        Action::ListOnFleaMarket { item, price } => apply_list_on_flea_market(state, &mut log, item, *price),
        Action::BuyFromFleaMarket { listing_id } => apply_buy_from_flea_market(state, &mut log, listing_id),
        Action::CancelFleaMarketListing { listing_id } => apply_cancel_flea_market_listing(state, &mut log, listing_id),
        Action::StartExploration { territory_id, card_indices } => {
            apply_start_exploration(state, &mut log, territory_id, card_indices)
        }
        Action::CollectExploration { mission_id } => apply_collect_exploration(state, &mut log, mission_id),
        Action::DonateAlliance { food, wood, stone, iron } => {
            apply_donate_alliance(state, &mut log, *food, *wood, *stone, *iron)
        }
    };
    // ハンドラーが state.clone() で早期リターンしても push_log の内容が失われないよう、
    // 常に外側で管理している log を最終状態に反映する
    result.log = log;
    result
}

/// owned_cards と同じ並びの Vec から、削除インデックス（昇順）に合わせて要素を除去
fn remove_indices_from_parallel_vec<T: Clone>( vec: &mut Vec<T>, sorted_asc: &[usize]) {
    for &idx in sorted_asc.iter().rev() {
        if idx < vec.len() {
            vec.remove(idx);
        }
    }
}

/// KC準拠カード合成: 素材カードを消費してベースカードのスキルレベルアップ
fn apply_synthesize_card(
    state: &GameState,
    log: &mut Vec<String>,
    base_idx: usize,
    material_indices: &[usize],
) -> GameState {
    let mut owned_cards = state.owned_cards.clone();
    if base_idx >= owned_cards.len() { return state.clone(); }
    if material_indices.is_empty() { return state.clone(); }
    for &idx in material_indices {
        if idx >= owned_cards.len() || idx == base_idx { return state.clone(); }
    }

    let base_card_id = owned_cards[base_idx];
    let base_name = crate::cards::get_card(base_card_id)
        .map(|c| c.name.to_string())
        .unwrap_or_else(|| format!("カード#{}", base_card_id));

    let material_count = material_indices.len();
    let level_up = (material_count as u8).min(9);

    push_log(log, format!(
        "「{}」に素材{}枚を合成！スキルLv+{}",
        base_name, material_count, level_up
    ));

    let mut to_remove: Vec<usize> = material_indices.to_vec();
    to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for idx in to_remove {
        owned_cards.remove(idx);
    }

    let mut sorted_removals: Vec<usize> = material_indices.to_vec();
    sorted_removals.sort();
    let mut card_skill_levels = std::collections::HashMap::new();
    for (&old_idx, &levels) in &state.card_skill_levels {
        if material_indices.contains(&old_idx) { continue; }
        let shift = sorted_removals.iter().filter(|&&r| r < old_idx).count();
        card_skill_levels.insert(old_idx - shift, levels);
    }

    let new_base_idx = base_idx - sorted_removals.iter().filter(|&&r| r < base_idx).count();
    let levels = card_skill_levels.entry(new_base_idx).or_insert([0u8; 3]);
    for lv in levels.iter_mut() {
        *lv = (*lv + level_up).min(10);
    }

    let mut card_levels = state.card_levels.clone();
    let mut card_exp = state.card_exp.clone();
    let mut card_stamina = state.card_stamina.clone();
    if card_levels.len() == state.owned_cards.len() {
        remove_indices_from_parallel_vec(&mut card_levels, &sorted_removals);
    }
    if card_exp.len() == state.owned_cards.len() {
        remove_indices_from_parallel_vec(&mut card_exp, &sorted_removals);
    }
    if card_stamina.len() == state.owned_cards.len() {
        remove_indices_from_parallel_vec(&mut card_stamina, &sorted_removals);
    }

    let mut players = state.players.clone();
    if let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) {
        player.owned_cards = owned_cards.clone();
        player.card_skill_levels = card_skill_levels.clone();
        player.card_levels = card_levels.clone();
        player.card_exp = card_exp.clone();
        player.card_stamina = card_stamina.clone();
    }

    let mut next = state.clone();
    next.owned_cards = owned_cards.clone();
    next.card_skill_levels = card_skill_levels.clone();
    next.card_levels = card_levels;
    next.card_exp = card_exp;
    next.card_stamina = card_stamina;

    build_game_state(
        &next,
        state.turn,
        state.territories.clone(),
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        owned_cards,
        card_skill_levels,
    )
}

fn apply_create_alliance(state: &GameState, log: &mut Vec<String>, name: &str) -> GameState {
    if state.alliances.iter().any(|a| a.member_ids.contains(&DEFAULT_PLAYER_ID.to_string())) {
        push_log(log, "既に同盟に所属しています。".to_string());
        return state.clone();
    }
    let alliance_id = format!("alliance_{}", state.alliances.len() + 1);
    let alliance = crate::model::Alliance {
        id: alliance_id.clone(),
        name: name.to_string(),
        leader_id: DEFAULT_PLAYER_ID.to_string(),
        member_ids: vec![DEFAULT_PLAYER_ID.to_string()],
        territory_points: 0,
        level: 1,
        donated_total: 0,
        parent_alliance_id: None,
        child_alliance_ids: vec![],
    };
    let mut new_state = state.clone();
    new_state.alliances.push(alliance);
    push_log(log, format!("同盟「{}」を結成しました！", name));
    new_state.log = log.clone();
    new_state
}

fn apply_join_alliance(state: &GameState, log: &mut Vec<String>, alliance_id: &str) -> GameState {
    if state.alliances.iter().any(|a| a.member_ids.contains(&DEFAULT_PLAYER_ID.to_string())) {
        push_log(log, "既に同盟に所属しています。".to_string());
        return state.clone();
    }
    let mut new_state = state.clone();
    if let Some(alliance) = new_state.alliances.iter_mut().find(|a| a.id == alliance_id) {
        alliance.member_ids.push(DEFAULT_PLAYER_ID.to_string());
        push_log(log, format!("同盟「{}」に参加しました！", alliance.name));
    } else {
        push_log(log, "同盟が見つかりません。".to_string());
    }
    new_state.log = log.clone();
    new_state
}

fn apply_leave_alliance(state: &GameState, log: &mut Vec<String>) -> GameState {
    let mut new_state = state.clone();
    let player_id = DEFAULT_PLAYER_ID.to_string();
    if let Some(alliance) = new_state.alliances.iter_mut().find(|a| a.member_ids.contains(&player_id)) {
        alliance.member_ids.retain(|id| id != &player_id);
        push_log(log, format!("同盟「{}」を脱退しました。", alliance.name));
        if alliance.member_ids.is_empty() {
            let alliance_name = alliance.name.clone();
            new_state.alliances.retain(|a| !a.member_ids.is_empty());
            push_log(log, format!("同盟「{}」は解散しました。", alliance_name));
        } else if alliance.leader_id == player_id {
            alliance.leader_id = alliance.member_ids[0].clone();
            push_log(log, format!("リーダーが{}に引き継がれました。", alliance.leader_id));
        }
    }
    new_state.log = log.clone();
    new_state
}

fn apply_build_base(state: &GameState, log: &mut Vec<String>, territory_id: &str) -> GameState {
    let mut territories = state.territories.clone();
    let idx = match get_territory_index(&territories, territory_id) {
        Some(i) => i,
        None => return state.clone(),
    };
    if territories[idx].owner_id.as_deref() != Some(DEFAULT_PLAYER_ID) {
        return state.clone();
    }
    if territories[idx].is_base || is_home_territory(territory_id) {
        return state.clone();
    }
    let mut players = state.players.clone();
    if let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) {
        let cost_food = 200u64;
        let cost_wood = 300u64;
        let cost_stone = 200u64;
        let cost_iron = 100u64;
        if player.resources.food < cost_food
            || player.resources.wood < cost_wood
            || player.resources.stone < cost_stone
            || player.resources.iron < cost_iron
        {
            push_log(log, "資源が足りません。".to_string());
            return state.clone();
        }
        player.resources.food -= cost_food;
        player.resources.wood -= cost_wood;
        player.resources.stone -= cost_stone;
        player.resources.iron -= cost_iron;
    }
    territories[idx].is_base = true;
    let lvl = territories[idx].level as u32;
    territories[idx].max_durability = 400u32.saturating_add(lvl.saturating_mul(120));
    territories[idx].durability = territories[idx].max_durability;
    territories[idx].tower_level = 1;
    let name = territory_name(&territories, territory_id).to_string();
    push_log(log, format!("{}に前線基地を建設しました！", name));

    build_game_state(
        state,
        state.turn,
        territories,
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    )
}

fn apply_end_turn_action(state: &GameState, log: &mut Vec<String>) -> GameState {
    push_log(log, format!("--- ターン {} 終了 ---", state.turn));
    let mut players = state.players.clone();
    if let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) {
        let fac_b = crate::facilities::calculate_facility_bonuses(&player.facilities);
        let bonus = 5_u32.saturating_add(fac_b.stamina_recovery_bonus);
        let max_stamina = 120_u32;
        while player.card_stamina.len() < player.owned_cards.len() {
            player.card_stamina.push(max_stamina);
        }
        for st in player.card_stamina.iter_mut() {
            *st = (*st).saturating_add(bonus).min(max_stamina);
        }
    }
    build_game_state(
        state,
        state.turn + 1,
        state.territories.clone(),
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    )
}

fn apply_deploy_action(
    state: &GameState,
    log: &mut Vec<String>,
    territory_id: &str,
    count: u32,
    monsters_per_body: &Option<Vec<u32>>,
    deploy_body_names: &Option<Vec<String>>,
) -> GameState {
    if is_home_territory(territory_id) {
        return state.clone();
    }

    let mut territories = state.territories.clone();
    let Some(idx) = get_territory_index(&territories, territory_id) else {
        return state.clone();
    };
    if !can_receive_reinforcement(&territories, &state.deployable_owner_ids, territory_id) {
        return state.clone();
    }
    if count == 0 || count > 100 {
        return state.clone();
    }

    let name = territory_name(&territories, territory_id).to_string();
    territories[idx].troops += count;

    let reinforcement_monster_counts: Vec<u32> = monsters_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![1u32; count as usize]);
    if let Some(ref mut values) = territories[idx].body_monster_counts {
        values.extend(reinforcement_monster_counts.iter());
    } else {
        let existing = territories[idx].troops.saturating_sub(count) as usize;
        let mut next_monster_counts = vec![1u32; existing];
        next_monster_counts.extend(reinforcement_monster_counts.iter());
        territories[idx].body_monster_counts = Some(next_monster_counts);
    }

    let reinforcement_names: Vec<String> = deploy_body_names
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| (1..=count as usize).map(|i| format!("援軍{}", i)).collect());
    if let Some(ref mut values) = territories[idx].body_names {
        values.extend(reinforcement_names.iter().cloned());
    } else {
        let existing = territories[idx].troops.saturating_sub(count) as usize;
        let mut next_names: Vec<String> = (1..=existing).map(|i| format!("守備{}", i)).collect();
        next_names.extend(reinforcement_names.iter().cloned());
        territories[idx].body_names = Some(next_names);
    }

    let total_monster_count: u32 = reinforcement_monster_counts.iter().sum();
    push_log(
        log,
        format!("ターン{}: {}に魔獣数{}（合計{}）を増援した。", state.turn, name, count, total_monster_count),
    );

    build_game_state(
        state,
        state.turn,
        territories,
        log.clone(),
        state.players.clone(),
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    )
}

fn assign_positions(chars: &mut [CombatCharacter]) {
    let len = chars.len();
    if len == 0 {
        return;
    }
    for c in chars.iter_mut() {
        c.position = crate::skills::Position::Front;
    }
    chars[len - 1].position = crate::skills::Position::Leader;
    if len >= 3 {
        chars[len - 2].position = crate::skills::Position::Back;
    }
}

/// 同族ボーナス: 同一種族2体で攻防+5%、3体以上で+10%
fn apply_race_bonus(chars: &mut [CombatCharacter], log: &mut Vec<String>) {
    use std::collections::HashMap;
    let mut race_counts: HashMap<crate::cards::Race, usize> = HashMap::new();
    for c in chars.iter() {
        if let Some(r) = c.race {
            *race_counts.entry(r).or_insert(0) += 1;
        }
    }
    let mut bonus_races: Vec<(crate::cards::Race, f32)> = Vec::new();
    for (race, count) in &race_counts {
        if *count >= 3 {
            bonus_races.push((*race, 0.10));
        } else if *count >= 2 {
            bonus_races.push((*race, 0.05));
        }
    }
    if bonus_races.is_empty() {
        return;
    }
    for c in chars.iter_mut() {
        if let Some(r) = c.race {
            if let Some((_, frac)) = bonus_races.iter().find(|(br, _)| *br == r) {
                let atk_bonus = (c.attack as f32 * frac).ceil() as u32;
                let def_bonus = (c.defense as f32 * frac).ceil() as u32;
                c.attack += atk_bonus;
                c.defense += def_bonus;
            }
        }
    }
    for (race, frac) in &bonus_races {
        let race_name = match race {
            crate::cards::Race::Beast => "獣族",
            crate::cards::Race::Demihuman => "亜人族",
            crate::cards::Race::Demon => "魔族",
            crate::cards::Race::Dragon => "龍族",
            crate::cards::Race::Giant => "巨人族",
            crate::cards::Race::Spirit => "精霊族",
            crate::cards::Race::Undead => "不死族",
        };
        let pct = (*frac * 100.0) as u32;
        push_log(
            log,
            format!("◆ {}の同族ボーナス発動（攻撃・防御+{}%）", race_name, pct),
        );
    }
}

/// KC準拠: 種族ティア（巨人・魔・龍 > 亜人・不死・精霊 > 獣）
fn race_combat_tier(r: crate::cards::Race) -> u8 {
    use crate::cards::Race::*;
    match r {
        Giant | Demon | Dragon => 2,
        Demihuman | Undead | Spirit => 1,
        Beast => 0,
    }
}

/// 攻撃側が上位ティアほど与ダメージ増、下位なら減（1段階あたり±12%）
fn race_matchup_damage_multiplier(att: Option<crate::cards::Race>, def: Option<crate::cards::Race>) -> f32 {
    match (att, def) {
        (Some(a), Some(d)) => {
            let diff = race_combat_tier(a) as i32 - race_combat_tier(d) as i32;
            1.0 + (diff as f32) * 0.12
        }
        _ => 1.0,
    }
}

/// KC準拠の最低ダメージ（魔獣数に応じた段階式、上限は max_dmg と整合するよう cap）
fn kc_minimum_damage(mc: f32) -> f32 {
    let mc = mc.max(1e-6);
    let cap = mc * 1.1;
    let raw_min = if mc <= 100.0 {
        (mc * 0.10).max(5.0)
    } else if mc <= 1000.0 {
        mc * 0.10
    } else if mc <= 5000.0 {
        let t = (mc - 1000.0) / 4000.0;
        mc * (0.10 - t * 0.05)
    } else if mc <= 9999.0 {
        let t = (mc - 5000.0) / 4999.0;
        let p9999 = 326.0_f32 / 9999.0;
        mc * (0.05 - t * (0.05 - p9999))
    } else {
        mc * (326.0_f32 / 9999.0)
    };
    raw_min.min(cap)
}

/// KC準拠のターゲット選択（射程1:前→中→指揮 / 射程2:前+中→指揮 / 射程3:指揮→中→前、挑発優先）
fn find_target(attacker_range: u8, enemies: &[CombatCharacter]) -> Option<usize> {
    use crate::skills::Position;

    let alive_front: Vec<usize> = enemies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_alive && c.position == Position::Front)
        .map(|(i, _)| i)
        .collect();
    let alive_back: Vec<usize> = enemies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_alive && c.position == Position::Back)
        .map(|(i, _)| i)
        .collect();
    let alive_leader: Vec<usize> = enemies
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_alive && c.position == Position::Leader)
        .map(|(i, _)| i)
        .collect();

    let mut pool: Vec<usize> = match attacker_range {
        1 => {
            if !alive_front.is_empty() {
                alive_front
            } else if !alive_back.is_empty() {
                alive_back
            } else {
                alive_leader
            }
        }
        2 => {
            let merged: Vec<usize> = alive_front.iter().chain(alive_back.iter()).copied().collect();
            if !merged.is_empty() {
                merged
            } else {
                alive_leader
            }
        }
        _ => {
            if !alive_leader.is_empty() {
                alive_leader
            } else if !alive_back.is_empty() {
                alive_back
            } else {
                alive_front
            }
        }
    };

    if pool.is_empty() {
        return None;
    }

    let taunted: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|&i| enemies[i].is_taunting())
        .collect();
    if !taunted.is_empty() {
        pool = taunted;
    }

    pool.choose(&mut rand::thread_rng()).copied()
}

/// `exclude` のインデックスをターゲット候補から外す（混乱・味方誤射用）
fn find_target_excluding(
    attacker_range: u8,
    chars: &[CombatCharacter],
    exclude: Option<usize>,
) -> Option<usize> {
    use crate::skills::Position;

    let alive_front: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            c.is_alive
                && c.position == Position::Front
                && exclude.map(|e| e != *i).unwrap_or(true)
        })
        .map(|(i, _)| i)
        .collect();
    let alive_back: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            c.is_alive
                && c.position == Position::Back
                && exclude.map(|e| e != *i).unwrap_or(true)
        })
        .map(|(i, _)| i)
        .collect();
    let alive_leader: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(i, c)| {
            c.is_alive
                && c.position == Position::Leader
                && exclude.map(|e| e != *i).unwrap_or(true)
        })
        .map(|(i, _)| i)
        .collect();

    let mut pool: Vec<usize> = match attacker_range {
        1 => {
            if !alive_front.is_empty() {
                alive_front
            } else if !alive_back.is_empty() {
                alive_back
            } else {
                alive_leader
            }
        }
        2 => {
            let merged: Vec<usize> = alive_front.iter().chain(alive_back.iter()).copied().collect();
            if !merged.is_empty() {
                merged
            } else {
                alive_leader
            }
        }
        _ => {
            if !alive_leader.is_empty() {
                alive_leader
            } else if !alive_back.is_empty() {
                alive_back
            } else {
                alive_front
            }
        }
    };

    if pool.is_empty() {
        return None;
    }

    let taunted: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|&i| chars[i].is_taunting())
        .collect();
    if !taunted.is_empty() {
        pool = taunted;
    }

    pool.choose(&mut rand::thread_rng()).copied()
}

fn attack_mods_has_skill_damage(mods: &crate::skills::AttackModifiers) -> bool {
    (mods.damage_multiplier - 1.0).abs() > 1e-4
        || mods.damage_add > 1e-4
        || mods.true_damage > 1e-4
        || mods.percent_damage > 1e-4
        || mods.aoe_damage > 1e-4
        || mods.execute_threshold > 1e-4
}

/// 攻撃時スキルによる付与効果（状態異常・自己バフ・味方列・回復）
fn apply_on_attack_skill_followup(
    mods: &crate::skills::AttackModifiers,
    attackers: &mut [CombatCharacter],
    defenders: &mut [CombatCharacter],
    attacker_idx: usize,
    defender_idx: usize,
    log: &mut Vec<String>,
) {
    for status_effect in &mods.status_effects {
        if defender_idx < defenders.len() && defenders[defender_idx].is_alive {
            apply_effect_to_character(status_effect, &mut defenders[defender_idx], log);
        }
    }
    for self_effect in &mods.self_effects {
        if attacker_idx < attackers.len() {
            apply_effect_to_character(self_effect, &mut attackers[attacker_idx], log);
        }
    }
    for ally_effect in &mods.ally_effects {
        for ally in attackers.iter_mut() {
            if ally.is_alive {
                apply_effect_to_character(ally_effect, ally, log);
            }
        }
    }
    for heal_effect in &mods.heal_effects {
        if let Some(lowest) = attackers.iter_mut().filter(|c| c.is_alive).min_by(|a, b| {
            a.current_monster_count
                .partial_cmp(&b.current_monster_count)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            apply_effect_to_character(heal_effect, lowest, log);
        }
    }
}

/// 同一陣営内の誤射時（魅了・混乱）のスキル付与
fn apply_on_attack_skill_followup_one_team(
    mods: &crate::skills::AttackModifiers,
    team: &mut [CombatCharacter],
    attacker_idx: usize,
    defender_idx: usize,
    log: &mut Vec<String>,
) {
    for status_effect in &mods.status_effects {
        if defender_idx < team.len() && team[defender_idx].is_alive {
            apply_effect_to_character(status_effect, &mut team[defender_idx], log);
        }
    }
    for self_effect in &mods.self_effects {
        if attacker_idx < team.len() {
            apply_effect_to_character(self_effect, &mut team[attacker_idx], log);
        }
    }
    for ally_effect in &mods.ally_effects {
        for ally in team.iter_mut() {
            if ally.is_alive {
                apply_effect_to_character(ally_effect, ally, log);
            }
        }
    }
    for heal_effect in &mods.heal_effects {
        if let Some(lowest) = team.iter_mut().filter(|c| c.is_alive).min_by(|a, b| {
            a.current_monster_count
                .partial_cmp(&b.current_monster_count)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            apply_effect_to_character(heal_effect, lowest, log);
        }
    }
}

fn compute_net_attack_damage(
    attacker: &CombatCharacter,
    defender: &CombatCharacter,
    attack_mods: &crate::skills::AttackModifiers,
    log: &mut Vec<String>,
) -> f32 {
    let atk_stat = attacker.attack.max(attacker.intelligence) as f32 * attacker.attack_buff_multiplier();
    let use_magic = attacker.intelligence > attacker.attack;
    let def_stat = if attack_mods.ignore_defense {
        0.0
    } else if use_magic {
        defender.magic_defense as f32
            * (1.0 - defender.damage_reduction)
            * defender.defense_buff_multiplier()
    } else {
        defender.defense as f32
            * (1.0 - defender.damage_reduction)
            * defender.defense_buff_multiplier()
    };
    let mc = attacker.current_monster_count;
    let ratio = (atk_stat / def_stat.max(1.0)).clamp(0.3, 1.1);
    let mut raw_damage = ratio
        * mc
        * attacker.damage_multiplier
        * attacker.outgoing_damage_multiplier();
    raw_damage = raw_damage * attack_mods.damage_multiplier + attack_mods.damage_add;
    raw_damage += attack_mods.true_damage;

    if attack_mods.percent_damage > 0.0 {
        let percent_dmg = defender.current_monster_count * attack_mods.percent_damage;
        raw_damage += percent_dmg;
        push_log(log, format!("割合ダメージ+{:.0}", percent_dmg));
    }

    let vulnerability = defender.get_vulnerability();
    let mark_damage = defender.get_mark_damage();
    raw_damage = raw_damage * (1.0 + vulnerability) + mark_damage;
    raw_damage *= race_matchup_damage_multiplier(attacker.race, defender.race);

    let min_dmg = kc_minimum_damage(mc);
    let max_dmg = mc * 1.1;
    let (low, high) = if min_dmg <= max_dmg {
        (min_dmg, max_dmg)
    } else {
        (max_dmg, max_dmg)
    };
    raw_damage.clamp(low, high)
}

#[allow(clippy::too_many_arguments)]
fn apply_attack_action(
    state: &GameState,
    log: &mut Vec<String>,
    from_territory_id: &str,
    to_territory_id: &str,
    count: u32,
    monsters_per_body: &Option<Vec<u32>>,
    our_body_names: &Option<Vec<String>>,
    attack_unit_name: &Option<String>,
    speed_per_body: &Option<Vec<u32>>,
    skills_per_body: &Option<Vec<SkillData>>,
    stats_per_body: &Option<Vec<CardStats>>,
    owned_card_indices: &Option<Vec<usize>>,
    dev_auto_win: bool,
) -> GameState {
    let mut territories = state.territories.clone();
    let from_idx = match get_territory_index(&territories, from_territory_id) {
        Some(index) => index,
        None => return state.clone(),
    };
    let to_idx = match get_territory_index(&territories, to_territory_id) {
        Some(index) => index,
        None => return state.clone(),
    };
    let attack_target_neutral = territories[to_idx].owner_id.is_none();
    let home_id = home_territory_id();
    if get_territory_index(&territories, &home_id).is_none() {
        return state.clone();
    }
    if from_idx == to_idx {
        return state.clone();
    }
    if is_home_territory(to_territory_id) {
        push_log(log, "本拠地は攻撃できません。".to_string());
        return state.clone();
    }
    if territories[from_idx].owner_id.as_deref() != Some("player") {
        push_log(log, format!("出撃元({})を所有していません。", from_territory_id));
        return state.clone();
    }
    if territories[from_idx].troops < count || count == 0 {
        push_log(log, format!(
            "出撃元({})の兵力が足りません（必要{}, 現在{}）。",
            from_territory_id, count, territories[from_idx].troops
        ));
        return state.clone();
    }
    let base_owners = attack_base_owner_ids(state, DEFAULT_PLAYER_ID);
    if !is_attackable_target(&territories, to_territory_id, &base_owners) {
        push_log(log, format!("{}は攻撃対象外です（隣接領地なし）。", to_territory_id));
        return state.clone();
    }
    if !territories_are_adjacent(from_territory_id, to_territory_id) {
        push_log(log, format!("{}と{}は隣接していません。", from_territory_id, to_territory_id));
        return state.clone();
    }

    let our_body_monster_counts: Vec<u32> = monsters_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![1u32; count as usize]);
    let our_speeds: Vec<u32> = speed_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![5u32; count as usize]);
    let our_names: Vec<String> = our_body_names
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| (1..=count as usize).map(|i| format!("味方ユニット{}", i)).collect());
    let our_skills: Vec<SkillData> = skills_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![SkillData::default(); count as usize]);
    let our_stats: Vec<CardStats> = stats_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![CardStats::default(); count as usize]);

    let facility_bonuses = crate::facilities::calculate_facility_bonuses(&state.facilities);
    let cost_cap = state
        .players
        .get(DEFAULT_PLAYER_ID)
        .map(|p| p.unit_cost_cap)
        .unwrap_or(state.unit_cost_cap)
        + facility_bonuses.unit_cost_cap_bonus;
    let total_cost: f32 = if our_stats.len() == count as usize {
        our_stats.iter().map(|s| s.cost).sum()
    } else {
        1.5 * count as f32
    };
    if total_cost > cost_cap + 0.0001 {
        push_log(
            log,
            format!(
                "ユニットコスト上限({:.1})を超えています（編成コスト合計{:.1}）。",
                cost_cap, total_cost
            ),
        );
        return state.clone();
    }

    const STAMINA_ATTACK: u32 = 25;

    let mut working_players = state.players.clone();
    crate::model::merge_legacy_into_working_players(&state.owned_cards, &mut working_players);

    if let Some(ref oci) = owned_card_indices {
        if oci.len() != count as usize {
            return state.clone();
        }
        let Some(player) = working_players.get(DEFAULT_PLAYER_ID) else {
            return state.clone();
        };
        for &i in oci {
            if i >= player.owned_cards.len() {
                push_log(log, "無効なカードインデックスです。".to_string());
                return state.clone();
            }
        }
        for &i in oci {
            let st = player.card_stamina.get(i).copied().unwrap_or(120);
            if st < STAMINA_ATTACK {
                push_log(log, "スタミナが足りないカードが含まれています。".to_string());
                return state.clone();
            }
        }
    }
    if let Some(ref oci) = owned_card_indices {
        if let Some(player) = working_players.get_mut(DEFAULT_PLAYER_ID) {
            while player.card_stamina.len() < player.owned_cards.len() {
                player.card_stamina.push(120);
            }
            for &i in oci {
                player.card_stamina[i] = player.card_stamina[i].saturating_sub(STAMINA_ATTACK);
            }
        }
    }

    let remaining_from_troops = territories[from_idx].troops.saturating_sub(count);
    territories[from_idx].troops = remaining_from_troops;
    if let Some(ref mut bm) = territories[from_idx].body_monster_counts {
        while bm.len() > remaining_from_troops as usize {
            bm.pop();
        }
    }
    if let Some(ref mut bn) = territories[from_idx].body_names {
        while bn.len() > remaining_from_troops as usize {
            bn.pop();
        }
    }

    let from_name = territory_name(&territories, from_territory_id).to_string();
    let to_name = territory_name(&territories, to_territory_id).to_string();
    let to_troops = territories[to_idx].troops;

    let mut our_chars: Vec<CombatCharacter> = our_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let stats = our_stats.get(index).cloned().unwrap_or_default();
            let base_monster_count = if stats.monster_count > 0 {
                stats.monster_count
            } else {
                *our_body_monster_counts.get(index).unwrap_or(&1)
            };
            let base_speed = if stats.speed > 0 { stats.speed } else { *our_speeds.get(index).unwrap_or(&5) };
            let attack = if stats.attack > 0 { stats.attack } else { 5 };
            let intelligence = if stats.intelligence > 0 { stats.intelligence } else { 5 };
            let defense = if stats.defense > 0 { stats.defense } else { 3 };
            let magic_defense = if stats.magic_defense > 0 { stats.magic_defense } else { 3 };
            let skills = our_skills.get(index).cloned().unwrap_or_default();

            let range = if stats.range > 0 { stats.range } else { 1 };
            let boosted_monster_count =
                crate::facilities::apply_monster_bonus(base_monster_count, &facility_bonuses);
            let boosted_speed = base_speed + facility_bonuses.speed_bonus;

            let mut ch = CombatCharacter::with_stats(
                index,
                name.clone(),
                boosted_monster_count,
                boosted_speed,
                attack,
                intelligence,
                defense,
                magic_defense,
                skills,
            );
            ch.range = range;
            ch.occupation_power = stats.occupation_power;
            let card_name_trimmed = name.trim_end_matches(|c: char| c == 'A' || c == 'B' || c == 'C');
            if let Some(card) = crate::cards::get_card_by_name(card_name_trimmed) {
                ch.race = Some(card.race);
                if ch.occupation_power == 0 {
                    ch.occupation_power = card.stats.occupation_power;
                }
            } else if ch.occupation_power == 0 {
                ch.occupation_power = crate::cards::CardStats::default().occupation_power;
            }
            ch
        })
        .collect();
    assign_positions(&mut our_chars);
    apply_race_bonus(&mut our_chars, log);

    let enemy_monster_counts: Vec<u32> = territories[to_idx]
        .body_monster_counts
        .clone()
        .filter(|values| values.len() == to_troops as usize)
        .unwrap_or_else(|| vec![1u32; to_troops as usize]);
    let enemy_names: Vec<String> = territories[to_idx]
        .body_names
        .clone()
        .filter(|values| values.len() == to_troops as usize)
        .unwrap_or_else(|| (1..=to_troops as usize).map(|i| format!("敵ユニット{}", i)).collect());

    let mut enemy_chars: Vec<CombatCharacter> = enemy_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let monster_count = *enemy_monster_counts.get(index).unwrap_or(&1);
            let card_name = name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C');
            if let Some(card) = crate::cards::get_card_by_name(card_name) {
                let mut ch = CombatCharacter::with_stats(
                    index + 100,
                    name.clone(),
                    monster_count,
                    card.stats.speed,
                    card.stats.attack,
                    card.stats.intelligence,
                    card.stats.defense,
                    card.stats.magic_defense,
                    crate::cards::get_card_skills(card.id),
                );
                ch.range = card.stats.range.max(1);
                ch.race = Some(card.race);
                ch.occupation_power = card.stats.occupation_power;
                ch
            } else {
                CombatCharacter::new(index + 100, name.clone(), monster_count, 5, SkillData::default())
            }
        })
        .collect();
    assign_positions(&mut enemy_chars);
    apply_race_bonus(&mut enemy_chars, log);

    if dev_auto_win {
        let max_enemy = enemy_chars.iter().map(|character| character.base_monster_count).max().unwrap_or(1);
        for character in our_chars.iter_mut() {
            character.current_monster_count = (max_enemy + 1) as f32;
        }
    }

    let attacker_label = attack_unit_name.as_deref().unwrap_or(from_name.as_str());
    let coords_str = parse_territory_coords(to_territory_id)
        .map(|(col, row)| format!("<{},{}>", col, row))
        .unwrap_or_default();
    push_log(
        log,
        format!("【{}{}侵攻戦】{}が{}へ侵攻開始", to_name, coords_str, attacker_label, to_name),
    );

    let total_waves = if territories[to_idx].ruin.is_some() {
        2u32
    } else {
        wave_count_for_level(territories[to_idx].level)
    };

    'waves: for wave in 1..=total_waves {
    if wave > 1 {
        if !our_chars.iter().any(|c| c.is_alive) { break 'waves; }
        let level = territories[to_idx].level;
        let (_cnt, monster_counts, names) = generate_neutral_enemies(level);
        enemy_chars.clear();
        let enemy_names_wave: Vec<String> = names;
        let enemy_monster_counts_wave: Vec<u32> = monster_counts;
        for (i, name) in enemy_names_wave.iter().enumerate() {
            let monster_count = *enemy_monster_counts_wave.get(i).unwrap_or(&1);
            let card_name = name.trim_end_matches(|c: char| c == 'A' || c == 'B' || c == 'C');
            if let Some(card) = crate::cards::get_card_by_name(card_name) {
                let mut ch = CombatCharacter::with_stats(
                    i + 100, name.clone(), monster_count, card.stats.speed,
                    card.stats.attack, card.stats.intelligence, card.stats.defense, card.stats.magic_defense,
                    crate::cards::get_card_skills(card.id),
                );
                ch.range = card.stats.range.max(1);
                ch.race = Some(card.race);
                ch.occupation_power = card.stats.occupation_power;
                enemy_chars.push(ch);
            } else {
                enemy_chars.push(CombatCharacter::new(
                    i + 100,
                    name.clone(),
                    monster_count,
                    5,
                    SkillData::default(),
                ));
            }
        }
        assign_positions(&mut enemy_chars);
        for c in our_chars.iter_mut() {
            c.status_effects.clear();
            c.damage_multiplier = 1.0;
            c.damage_reduction = 0.0;
            c.extra_attacks = 0;
        }
        push_log(log, format!("--- 第{}戦 ---", wave));
    }

    push_log(log, "--- スタートアップフェーズ ---".to_string());
    apply_battle_start_skills(&mut our_chars, &mut enemy_chars, log);

    push_log(log, "--- 戦闘フェーズ ---".to_string());

    let max_combat_turns: u32 = 8;
    let mut last_combat_turn: u32 = 0;
    'battle: for combat_turn in 1..=max_combat_turns {
        last_combat_turn = combat_turn;
        push_log(log, format!("--- Turn {} ---", combat_turn));

        let mut actors: Vec<(usize, bool)> = Vec::new();
        for (i, c) in our_chars.iter().enumerate() {
            if c.is_alive { actors.push((i, true)); }
        }
        for (i, c) in enemy_chars.iter().enumerate() {
            if c.is_alive { actors.push((i, false)); }
        }
        actors.sort_by(|a, b| {
            let sa = if a.1 {
                our_chars[a.0].turn_order_speed()
            } else {
                enemy_chars[a.0].turn_order_speed()
            };
            let sb = if b.1 {
                our_chars[b.0].turn_order_speed()
            } else {
                enemy_chars[b.0].turn_order_speed()
            };
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut rng = rand::thread_rng();
        let mut start = 0;
        while start < actors.len() {
            let sp = if actors[start].1 {
                our_chars[actors[start].0].turn_order_speed()
            } else {
                enemy_chars[actors[start].0].turn_order_speed()
            };
            let mut end = start + 1;
            while end < actors.len() {
                let sep = if actors[end].1 {
                    our_chars[actors[end].0].turn_order_speed()
                } else {
                    enemy_chars[actors[end].0].turn_order_speed()
                };
                if (sep - sp).abs() >= 1e-3 {
                    break;
                }
                end += 1;
            }
            if end - start > 1 {
                actors[start..end].shuffle(&mut rng);
            }
            start = end;
        }

        for &(actor_idx, is_ally) in &actors {
            if is_ally {
                if !our_chars[actor_idx].is_alive { continue; }

                if our_chars[actor_idx].is_disabled() {
                    push_log(log, format!("{}は行動不能！", our_chars[actor_idx].name));
                    continue;
                }

                let target_on_player_team = {
                    let ch = &our_chars[actor_idx];
                    if ch.is_charmed() {
                        push_log(log, format!("{}は魅了され味方を攻撃する！", ch.name));
                        true
                    } else {
                        let p = ch.confused_own_team_chance();
                        if p > 0.0 && rand::random::<f32>() < p {
                            push_log(log, format!("{}は混乱して味方を狙う！", ch.name));
                            true
                        } else {
                            false
                        }
                    }
                };
                let target_idx = if target_on_player_team {
                    match find_target_excluding(our_chars[actor_idx].range, &our_chars, Some(actor_idx)) {
                        Some(i) => i,
                        None => continue,
                    }
                } else {
                    match find_target(our_chars[actor_idx].range, &enemy_chars) {
                        Some(i) => i,
                        None => continue,
                    }
                };

                let attack_mods = if our_chars[actor_idx].is_silenced() {
                    push_log(log, format!("{}は沈黙中でスキル使用不可！", our_chars[actor_idx].name));
                    crate::skills::AttackModifiers::new()
                } else {
                    apply_attack_skills(&mut our_chars[actor_idx], log)
                };

                if attack_mods.skill_activated && !attack_mods_has_skill_damage(&attack_mods) {
                    if target_on_player_team {
                        apply_on_attack_skill_followup_one_team(
                            &attack_mods,
                            &mut our_chars,
                            actor_idx,
                            target_idx,
                            log,
                        );
                    } else {
                        apply_on_attack_skill_followup(
                            &attack_mods,
                            &mut our_chars,
                            &mut enemy_chars,
                            actor_idx,
                            target_idx,
                            log,
                        );
                    }
                    continue;
                }

                let evasion_rate = if target_on_player_team {
                    our_chars[target_idx].get_evasion_rate()
                } else {
                    enemy_chars[target_idx].get_evasion_rate()
                };
                if evasion_rate > 0.0 && rand::random::<f32>() < evasion_rate {
                    let def_nm = if target_on_player_team {
                        our_chars[target_idx].name.clone()
                    } else {
                        enemy_chars[target_idx].name.clone()
                    };
                    push_log(
                        log,
                        format!("{}の攻撃を{}が回避！", our_chars[actor_idx].name, def_nm),
                    );
                    continue;
                }

                if target_on_player_team {
                    if our_chars[target_idx].consume_invincible() {
                        push_log(
                            log,
                            format!("{}は無敵で攻撃を無効化！", our_chars[target_idx].name),
                        );
                        continue;
                    }
                } else if enemy_chars[target_idx].consume_invincible() {
                    push_log(
                        log,
                        format!("{}は無敵で攻撃を無効化！", enemy_chars[target_idx].name),
                    );
                    continue;
                }

                let our_name = our_chars[actor_idx].name.clone();
                let enemy_name = if target_on_player_team {
                    our_chars[target_idx].name.clone()
                } else {
                    enemy_chars[target_idx].name.clone()
                };

                let net_damage = {
                    let a = &our_chars[actor_idx];
                    let d = if target_on_player_team {
                        &our_chars[target_idx]
                    } else {
                        &enemy_chars[target_idx]
                    };
                    compute_net_attack_damage(a, d, &attack_mods, log)
                };

                push_log(log, format!(
                    "{}が{}に攻撃！（ダメージ{:.0}）",
                    our_name, enemy_name, net_damage
                ));

                if target_on_player_team {
                    let hp_ratio = our_chars[target_idx].current_monster_count
                        / our_chars[target_idx].base_monster_count as f32;
                    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
                        our_chars[target_idx].is_alive = false;
                        our_chars[target_idx].current_monster_count = 0.0;
                        push_log(log, format!("処刑発動！{}を即死させた！", enemy_name));
                    } else {
                        if attack_mods.aoe_damage > 0.0 {
                            push_log(log, format!("全体攻撃で味方全員に{:.0}ダメージ！", attack_mods.aoe_damage));
                            for ally in our_chars.iter_mut() {
                                if ally.is_alive {
                                    ally.current_monster_count -= attack_mods.aoe_damage;
                                    if ally.current_monster_count <= 0.0 {
                                        ally.is_alive = false;
                                        if !check_death_skills(ally, log) {
                                            push_log(log, format!("{}が全体攻撃で撃破されました。", ally.name));
                                        }
                                    }
                                }
                            }
                        }

                        let damage_after_shield =
                            our_chars[target_idx].absorb_damage_with_shield(net_damage);
                        if damage_after_shield > 0.0 {
                            our_chars[target_idx].current_monster_count -= damage_after_shield;
                        }

                        if our_chars[target_idx].current_monster_count <= 0.0 {
                            our_chars[target_idx].is_alive = false;
                            if !check_death_skills(&mut our_chars[target_idx], log) {
                                push_log(log, format!("{}が{}を撃破しました。", our_name, enemy_name));
                            }

                            if attack_mods.monster_steal > 0.0 {
                                our_chars[actor_idx].current_monster_count += attack_mods.monster_steal;
                                push_log(log, format!("{}が{:.0}魔獣数を奪取！", our_name, attack_mods.monster_steal));
                            }
                            if attack_mods.absorb_rate > 0.0 {
                                let absorb = damage_after_shield * attack_mods.absorb_rate;
                                our_chars[actor_idx].current_monster_count += absorb;
                                push_log(log, format!("{}が{:.0}魔獣数を吸収！", our_name, absorb));
                            }
                            if attack_mods.extra_attacks > 0 {
                                our_chars[actor_idx].extra_attacks += attack_mods.extra_attacks;
                                push_log(log, format!("{}が追加攻撃権を得た！", our_name));
                            }
                        } else if damage_after_shield <= 0.0 {
                            push_log(log, format!("{}のシールドがダメージを吸収！", enemy_name));
                        } else {
                            let reflect_rate = our_chars[target_idx].get_reflect_rate();
                            if reflect_rate > 0.0 {
                                let reflect_damage = net_damage * reflect_rate;
                                our_chars[actor_idx].current_monster_count -= reflect_damage;
                                push_log(log, format!("{}の反射で{:.0}ダメージ！", enemy_name, reflect_damage));
                                if our_chars[actor_idx].current_monster_count <= 0.0 {
                                    our_chars[actor_idx].is_alive = false;
                                    if !check_death_skills(&mut our_chars[actor_idx], log) {
                                        push_log(log, format!("{}が反射で撃破されました。", our_name));
                                    }
                                }
                            }
                            let counter_rate = our_chars[target_idx].get_counter_rate();
                            if counter_rate > 0.0
                                && rand::random::<f32>() < counter_rate
                                && our_chars[actor_idx].is_alive
                            {
                                let c_atk = our_chars[target_idx]
                                    .attack
                                    .max(our_chars[target_idx].intelligence) as f32
                                    * our_chars[target_idx].attack_buff_multiplier();
                                let c_def = our_chars[actor_idx]
                                    .defense
                                    .max(our_chars[actor_idx].magic_defense) as f32
                                    * our_chars[actor_idx].defense_buff_multiplier();
                                let c_mc = our_chars[target_idx].current_monster_count;
                                let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
                                let counter_raw = c_ratio
                                    * c_mc
                                    * 0.5
                                    * our_chars[target_idx].outgoing_damage_multiplier();
                                let c_min = kc_minimum_damage(c_mc) * 0.5;
                                let c_max = c_mc * 0.55;
                                let (cl, ch) = if c_min <= c_max {
                                    (c_min, c_max)
                                } else {
                                    (c_max, c_max)
                                };
                                let counter_dmg = counter_raw.clamp(cl, ch);
                                our_chars[actor_idx].current_monster_count -= counter_dmg;
                                push_log(log, format!("{}の反撃！{:.0}ダメージ！", enemy_name, counter_dmg));
                                if our_chars[actor_idx].current_monster_count <= 0.0 {
                                    our_chars[actor_idx].is_alive = false;
                                    if !check_death_skills(&mut our_chars[actor_idx], log) {
                                        push_log(log, format!("{}が反撃で撃破されました。", our_name));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let hp_ratio = enemy_chars[target_idx].current_monster_count
                        / enemy_chars[target_idx].base_monster_count as f32;
                    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
                        enemy_chars[target_idx].is_alive = false;
                        enemy_chars[target_idx].current_monster_count = 0.0;
                        push_log(log, format!("処刑発動！{}を即死させた！", enemy_name));
                    } else {
                        if attack_mods.aoe_damage > 0.0 {
                            push_log(log, format!("全体攻撃で敵全員に{:.0}ダメージ！", attack_mods.aoe_damage));
                            for enemy in enemy_chars.iter_mut() {
                                if enemy.is_alive {
                                    enemy.current_monster_count -= attack_mods.aoe_damage;
                                    if enemy.current_monster_count <= 0.0 {
                                        enemy.is_alive = false;
                                        push_log(log, format!("{}が全体攻撃で撃破されました。", enemy.name));
                                    }
                                }
                            }
                        }

                        let damage_after_shield =
                            enemy_chars[target_idx].absorb_damage_with_shield(net_damage);
                        if damage_after_shield > 0.0 {
                            enemy_chars[target_idx].current_monster_count -= damage_after_shield;
                        }

                        if enemy_chars[target_idx].current_monster_count <= 0.0 {
                            enemy_chars[target_idx].is_alive = false;
                            push_log(log, format!("{}が{}を撃破しました。", our_name, enemy_name));

                            if attack_mods.monster_steal > 0.0 {
                                our_chars[actor_idx].current_monster_count += attack_mods.monster_steal;
                                push_log(log, format!("{}が{:.0}魔獣数を奪取！", our_name, attack_mods.monster_steal));
                            }
                            if attack_mods.absorb_rate > 0.0 {
                                let absorb = damage_after_shield * attack_mods.absorb_rate;
                                our_chars[actor_idx].current_monster_count += absorb;
                                push_log(log, format!("{}が{:.0}魔獣数を吸収！", our_name, absorb));
                            }
                            if attack_mods.extra_attacks > 0 {
                                our_chars[actor_idx].extra_attacks += attack_mods.extra_attacks;
                                push_log(log, format!("{}が追加攻撃権を得た！", our_name));
                            }
                        } else if damage_after_shield <= 0.0 {
                            push_log(log, format!("{}のシールドがダメージを吸収！", enemy_name));
                        } else {
                            let reflect_rate = enemy_chars[target_idx].get_reflect_rate();
                            if reflect_rate > 0.0 {
                                let reflect_damage = net_damage * reflect_rate;
                                our_chars[actor_idx].current_monster_count -= reflect_damage;
                                push_log(log, format!("{}の反射で{:.0}ダメージ！", enemy_name, reflect_damage));
                                if our_chars[actor_idx].current_monster_count <= 0.0 {
                                    our_chars[actor_idx].is_alive = false;
                                    if !check_death_skills(&mut our_chars[actor_idx], log) {
                                        push_log(log, format!("{}が反射で撃破されました。", our_name));
                                    }
                                }
                            }
                            let counter_rate = enemy_chars[target_idx].get_counter_rate();
                            if counter_rate > 0.0
                                && rand::random::<f32>() < counter_rate
                                && our_chars[actor_idx].is_alive
                            {
                                let c_atk = enemy_chars[target_idx]
                                    .attack
                                    .max(enemy_chars[target_idx].intelligence) as f32
                                    * enemy_chars[target_idx].attack_buff_multiplier();
                                let c_def = our_chars[actor_idx]
                                    .defense
                                    .max(our_chars[actor_idx].magic_defense) as f32
                                    * our_chars[actor_idx].defense_buff_multiplier();
                                let c_mc = enemy_chars[target_idx].current_monster_count;
                                let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
                                let counter_raw = c_ratio
                                    * c_mc
                                    * 0.5
                                    * enemy_chars[target_idx].outgoing_damage_multiplier();
                                let c_min = kc_minimum_damage(c_mc) * 0.5;
                                let c_max = c_mc * 0.55;
                                let (cl, ch) = if c_min <= c_max {
                                    (c_min, c_max)
                                } else {
                                    (c_max, c_max)
                                };
                                let counter_dmg = counter_raw.clamp(cl, ch);
                                our_chars[actor_idx].current_monster_count -= counter_dmg;
                                push_log(log, format!("{}の反撃！{:.0}ダメージ！", enemy_name, counter_dmg));
                                if our_chars[actor_idx].current_monster_count <= 0.0 {
                                    our_chars[actor_idx].is_alive = false;
                                    if !check_death_skills(&mut our_chars[actor_idx], log) {
                                        push_log(log, format!("{}が反撃で撃破されました。", our_name));
                                    }
                                }
                            }
                        }
                    }
                }

                if target_on_player_team {
                    apply_on_attack_skill_followup_one_team(
                        &attack_mods,
                        &mut our_chars,
                        actor_idx,
                        target_idx,
                        log,
                    );
                } else {
                    apply_on_attack_skill_followup(
                        &attack_mods,
                        &mut our_chars,
                        &mut enemy_chars,
                        actor_idx,
                        target_idx,
                        log,
                    );
                }

            } else {
                if !enemy_chars[actor_idx].is_alive { continue; }

                if enemy_chars[actor_idx].is_disabled() {
                    push_log(log, format!("{}は行動不能！", enemy_chars[actor_idx].name));
                    continue;
                }

                let target_on_player_team = {
                    let ch = &enemy_chars[actor_idx];
                    if ch.is_charmed() {
                        push_log(log, format!("{}は魅了され味方を攻撃する！", ch.name));
                        false
                    } else {
                        let p = ch.confused_own_team_chance();
                        if p > 0.0 && rand::random::<f32>() < p {
                            push_log(log, format!("{}は混乱して味方を狙う！", ch.name));
                            false
                        } else {
                            true
                        }
                    }
                };
                let target_idx = if target_on_player_team {
                    match find_target(enemy_chars[actor_idx].range, &our_chars) {
                        Some(i) => i,
                        None => continue,
                    }
                } else {
                    match find_target_excluding(
                        enemy_chars[actor_idx].range,
                        &enemy_chars,
                        Some(actor_idx),
                    ) {
                        Some(i) => i,
                        None => continue,
                    }
                };

                let atk_name = enemy_chars[actor_idx].name.clone();
                let def_name = if target_on_player_team {
                    our_chars[target_idx].name.clone()
                } else {
                    enemy_chars[target_idx].name.clone()
                };

                let attack_mods = if enemy_chars[actor_idx].is_silenced() {
                    push_log(log, format!("{}は沈黙中でスキル使用不可！", enemy_chars[actor_idx].name));
                    crate::skills::AttackModifiers::new()
                } else {
                    apply_attack_skills(&mut enemy_chars[actor_idx], log)
                };

                if attack_mods.skill_activated && !attack_mods_has_skill_damage(&attack_mods) {
                    if target_on_player_team {
                        apply_on_attack_skill_followup(
                            &attack_mods,
                            &mut enemy_chars,
                            &mut our_chars,
                            actor_idx,
                            target_idx,
                            log,
                        );
                    } else {
                        apply_on_attack_skill_followup_one_team(
                            &attack_mods,
                            &mut enemy_chars,
                            actor_idx,
                            target_idx,
                            log,
                        );
                    }
                    continue;
                }

                let evasion = if target_on_player_team {
                    our_chars[target_idx].get_evasion_rate()
                } else {
                    enemy_chars[target_idx].get_evasion_rate()
                };
                if evasion > 0.0 && rand::random::<f32>() < evasion {
                    push_log(log, format!("{}の攻撃を{}が回避！", atk_name, def_name));
                    continue;
                }
                if target_on_player_team {
                    if our_chars[target_idx].consume_invincible() {
                        push_log(log, format!("{}は無敵で攻撃を無効化！", def_name));
                        continue;
                    }
                } else if enemy_chars[target_idx].consume_invincible() {
                    push_log(log, format!("{}は無敵で攻撃を無効化！", def_name));
                    continue;
                }

                let net_damage = {
                    let a = &enemy_chars[actor_idx];
                    let d = if target_on_player_team {
                        &our_chars[target_idx]
                    } else {
                        &enemy_chars[target_idx]
                    };
                    compute_net_attack_damage(a, d, &attack_mods, log)
                };
                push_log(log, format!("{}が{}に攻撃！（ダメージ{:.0}）", atk_name, def_name, net_damage));

                if target_on_player_team {
                    let hp_ratio = our_chars[target_idx].current_monster_count
                        / our_chars[target_idx].base_monster_count as f32;
                    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
                        our_chars[target_idx].is_alive = false;
                        our_chars[target_idx].current_monster_count = 0.0;
                        push_log(log, format!("処刑発動！{}を即死させた！", def_name));
                    } else {
                        if attack_mods.aoe_damage > 0.0 {
                            push_log(log, format!("全体攻撃でプレイヤー側全員に{:.0}ダメージ！", attack_mods.aoe_damage));
                            for ally in our_chars.iter_mut() {
                                if ally.is_alive {
                                    ally.current_monster_count -= attack_mods.aoe_damage;
                                    if ally.current_monster_count <= 0.0 {
                                        ally.is_alive = false;
                                        if !check_death_skills(ally, log) {
                                            push_log(log, format!("{}が全体攻撃で撃破されました。", ally.name));
                                        }
                                    }
                                }
                            }
                        }

                        let damage_after_shield =
                            our_chars[target_idx].absorb_damage_with_shield(net_damage);
                        if damage_after_shield > 0.0 {
                            our_chars[target_idx].current_monster_count -= damage_after_shield;
                        }

                        if our_chars[target_idx].current_monster_count <= 0.0 {
                            our_chars[target_idx].is_alive = false;
                            if !check_death_skills(&mut our_chars[target_idx], log) {
                                push_log(log, format!("{}が撃破されました。", def_name));
                            }

                            if attack_mods.monster_steal > 0.0 {
                                enemy_chars[actor_idx].current_monster_count += attack_mods.monster_steal;
                                push_log(log, format!("{}が{:.0}魔獣数を奪取！", atk_name, attack_mods.monster_steal));
                            }
                            if attack_mods.absorb_rate > 0.0 {
                                let absorb = damage_after_shield * attack_mods.absorb_rate;
                                enemy_chars[actor_idx].current_monster_count += absorb;
                                push_log(log, format!("{}が{:.0}魔獣数を吸収！", atk_name, absorb));
                            }
                            if attack_mods.extra_attacks > 0 {
                                enemy_chars[actor_idx].extra_attacks += attack_mods.extra_attacks;
                                push_log(log, format!("{}が追加攻撃権を得た！", atk_name));
                            }
                        } else if damage_after_shield <= 0.0 {
                            push_log(log, format!("{}のシールドがダメージを吸収！", def_name));
                        } else {
                            let reflect_rate = our_chars[target_idx].get_reflect_rate();
                            if reflect_rate > 0.0 {
                                let reflect_dmg = net_damage * reflect_rate;
                                enemy_chars[actor_idx].current_monster_count -= reflect_dmg;
                                push_log(log, format!("{}の反射で{:.0}ダメージ！", def_name, reflect_dmg));
                                if enemy_chars[actor_idx].current_monster_count <= 0.0 {
                                    enemy_chars[actor_idx].is_alive = false;
                                    push_log(log, format!("{}が反射で撃破されました。", atk_name));
                                }
                            }
                            let counter_rate = our_chars[target_idx].get_counter_rate();
                            if counter_rate > 0.0
                                && rand::random::<f32>() < counter_rate
                                && enemy_chars[actor_idx].is_alive
                            {
                                let c_atk = our_chars[target_idx]
                                    .attack
                                    .max(our_chars[target_idx].intelligence) as f32
                                    * our_chars[target_idx].attack_buff_multiplier();
                                let c_def = enemy_chars[actor_idx]
                                    .defense
                                    .max(enemy_chars[actor_idx].magic_defense) as f32
                                    * enemy_chars[actor_idx].defense_buff_multiplier();
                                let c_mc = our_chars[target_idx].current_monster_count;
                                let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
                                let counter_raw = c_ratio
                                    * c_mc
                                    * 0.5
                                    * our_chars[target_idx].outgoing_damage_multiplier();
                                let c_min = kc_minimum_damage(c_mc) * 0.5;
                                let c_max = c_mc * 0.55;
                                let (cl, ch) = if c_min <= c_max {
                                    (c_min, c_max)
                                } else {
                                    (c_max, c_max)
                                };
                                let counter_dmg = counter_raw.clamp(cl, ch);
                                enemy_chars[actor_idx].current_monster_count -= counter_dmg;
                                push_log(log, format!("{}の反撃！{:.0}ダメージ！", def_name, counter_dmg));
                                if enemy_chars[actor_idx].current_monster_count <= 0.0 {
                                    enemy_chars[actor_idx].is_alive = false;
                                    push_log(log, format!("{}が反撃で撃破されました。", atk_name));
                                }
                            }
                        }
                    }
                } else {
                    let hp_ratio = enemy_chars[target_idx].current_monster_count
                        / enemy_chars[target_idx].base_monster_count as f32;
                    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
                        enemy_chars[target_idx].is_alive = false;
                        enemy_chars[target_idx].current_monster_count = 0.0;
                        push_log(log, format!("処刑発動！{}を即死させた！", def_name));
                    } else {
                        if attack_mods.aoe_damage > 0.0 {
                            push_log(log, format!("全体攻撃で敵全員に{:.0}ダメージ！", attack_mods.aoe_damage));
                            for enemy in enemy_chars.iter_mut() {
                                if enemy.is_alive {
                                    enemy.current_monster_count -= attack_mods.aoe_damage;
                                    if enemy.current_monster_count <= 0.0 {
                                        enemy.is_alive = false;
                                        push_log(log, format!("{}が全体攻撃で撃破されました。", enemy.name));
                                    }
                                }
                            }
                        }

                        let damage_after_shield =
                            enemy_chars[target_idx].absorb_damage_with_shield(net_damage);
                        if damage_after_shield > 0.0 {
                            enemy_chars[target_idx].current_monster_count -= damage_after_shield;
                        }

                        if enemy_chars[target_idx].current_monster_count <= 0.0 {
                            enemy_chars[target_idx].is_alive = false;
                            push_log(log, format!("{}が{}を撃破しました。", atk_name, def_name));

                            if attack_mods.monster_steal > 0.0 {
                                enemy_chars[actor_idx].current_monster_count += attack_mods.monster_steal;
                                push_log(log, format!("{}が{:.0}魔獣数を奪取！", atk_name, attack_mods.monster_steal));
                            }
                            if attack_mods.absorb_rate > 0.0 {
                                let absorb = damage_after_shield * attack_mods.absorb_rate;
                                enemy_chars[actor_idx].current_monster_count += absorb;
                                push_log(log, format!("{}が{:.0}魔獣数を吸収！", atk_name, absorb));
                            }
                            if attack_mods.extra_attacks > 0 {
                                enemy_chars[actor_idx].extra_attacks += attack_mods.extra_attacks;
                                push_log(log, format!("{}が追加攻撃権を得た！", atk_name));
                            }
                        } else if damage_after_shield <= 0.0 {
                            push_log(log, format!("{}のシールドがダメージを吸収！", def_name));
                        } else {
                            let reflect_rate = enemy_chars[target_idx].get_reflect_rate();
                            if reflect_rate > 0.0 {
                                let reflect_dmg = net_damage * reflect_rate;
                                enemy_chars[actor_idx].current_monster_count -= reflect_dmg;
                                push_log(log, format!("{}の反射で{:.0}ダメージ！", def_name, reflect_dmg));
                                if enemy_chars[actor_idx].current_monster_count <= 0.0 {
                                    enemy_chars[actor_idx].is_alive = false;
                                    push_log(log, format!("{}が反射で撃破されました。", atk_name));
                                }
                            }
                            let counter_rate = enemy_chars[target_idx].get_counter_rate();
                            if counter_rate > 0.0
                                && rand::random::<f32>() < counter_rate
                                && enemy_chars[actor_idx].is_alive
                            {
                                let c_atk = enemy_chars[target_idx]
                                    .attack
                                    .max(enemy_chars[target_idx].intelligence) as f32
                                    * enemy_chars[target_idx].attack_buff_multiplier();
                                let c_def = enemy_chars[actor_idx]
                                    .defense
                                    .max(enemy_chars[actor_idx].magic_defense) as f32
                                    * enemy_chars[actor_idx].defense_buff_multiplier();
                                let c_mc = enemy_chars[target_idx].current_monster_count;
                                let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
                                let counter_raw = c_ratio
                                    * c_mc
                                    * 0.5
                                    * enemy_chars[target_idx].outgoing_damage_multiplier();
                                let c_min = kc_minimum_damage(c_mc) * 0.5;
                                let c_max = c_mc * 0.55;
                                let (cl, ch) = if c_min <= c_max {
                                    (c_min, c_max)
                                } else {
                                    (c_max, c_max)
                                };
                                let counter_dmg = counter_raw.clamp(cl, ch);
                                enemy_chars[actor_idx].current_monster_count -= counter_dmg;
                                push_log(log, format!("{}の反撃！{:.0}ダメージ！", def_name, counter_dmg));
                                if enemy_chars[actor_idx].current_monster_count <= 0.0 {
                                    enemy_chars[actor_idx].is_alive = false;
                                    push_log(log, format!("{}が反撃で撃破されました。", atk_name));
                                }
                            }
                        }
                    }
                }

                if target_on_player_team {
                    apply_on_attack_skill_followup(
                        &attack_mods,
                        &mut enemy_chars,
                        &mut our_chars,
                        actor_idx,
                        target_idx,
                        log,
                    );
                } else {
                    apply_on_attack_skill_followup_one_team(
                        &attack_mods,
                        &mut enemy_chars,
                        actor_idx,
                        target_idx,
                        log,
                    );
                }
            }

            let enemy_leader_dead = enemy_chars.iter().any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
            let ally_leader_dead = our_chars.iter().any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
            if enemy_leader_dead || enemy_chars.iter().all(|c| !c.is_alive) { break 'battle; }
            if ally_leader_dead || our_chars.iter().all(|c| !c.is_alive) { break 'battle; }
        }

        for c in our_chars.iter_mut() {
            if c.is_alive { c.process_turn_effects(log); }
        }
        for c in enemy_chars.iter_mut() {
            if c.is_alive { c.process_turn_effects(log); }
        }

        let enemy_leader_dead = enemy_chars.iter().any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
        let ally_leader_dead = our_chars.iter().any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
        if enemy_leader_dead || enemy_chars.iter().all(|c| !c.is_alive) { break 'battle; }
        if ally_leader_dead || our_chars.iter().all(|c| !c.is_alive) { break 'battle; }
    }

    // 本当にターン上限まで泥沼だったときだけ防衛勝利。早期終了で両軍残存（例: 指揮官のみ撃破）はここに来ない。
    let enemy_commander_dead = enemy_chars
        .iter()
        .any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
    if our_chars.iter().any(|c| c.is_alive)
        && enemy_chars.iter().any(|c| c.is_alive)
        && last_combat_turn >= max_combat_turns
        && !enemy_commander_dead
    {
        push_log(log, "8ターン経過。防衛側の勝利。".to_string());
        break 'waves;
    }

    if !our_chars.iter().any(|c| c.is_alive) { break 'waves; }
    } // end of 'waves loop

    let surviving_allies: Vec<&CombatCharacter> = our_chars.iter().filter(|character| character.is_alive).collect();
    let surviving_enemies: Vec<&CombatCharacter> = enemy_chars.iter().filter(|character| character.is_alive).collect();

    let waves_cleared = surviving_enemies.is_empty();
    let enemy_leader_defeated = enemy_chars
        .iter()
        .any(|c| c.position == crate::skills::Position::Leader && !c.is_alive);
    let neutral_won_by_commander = attack_target_neutral
        && !surviving_allies.is_empty()
        && enemy_leader_defeated;
    let wave_won_for_occ = waves_cleared || neutral_won_by_commander;

    let mut conquered = false;
    let mut partial_occupation = false;
    if wave_won_for_occ {
        let occ: u32 = surviving_allies.iter().map(|c| c.occupation_power).sum();
        let max_d = territories[to_idx].max_durability;
        if max_d == 0 {
            conquered = true;
        } else {
            territories[to_idx].durability = territories[to_idx].durability.saturating_sub(occ);
            if territories[to_idx].durability == 0 {
                conquered = true;
            } else {
                partial_occupation = true;
                push_log(
                    log,
                    format!(
                        "敵を撃破したが{}は耐久{}が残り、占領には至らなかった。",
                        to_name, territories[to_idx].durability
                    ),
                );
                let level = territories[to_idx].level;
                let (t_troops, t_mc, t_names) = generate_neutral_enemies(level);
                territories[to_idx].troops = t_troops;
                territories[to_idx].body_monster_counts = Some(t_mc);
                territories[to_idx].body_names = Some(t_names);
            }
        }
    }

    let mut new_inventory = state.inventory.clone();
    let mut new_owned_cards = state.owned_cards.clone();

    if conquered {
        territories[to_idx].owner_id = Some("player".to_string());
        territories[to_idx].max_durability = 0;
        territories[to_idx].durability = 0;
        territories[to_idx].tower_level = 0;
        let occupying: Vec<u32> = if surviving_allies.is_empty() {
            vec![1u32]
        } else {
            surviving_allies.iter().map(|character| character.effective_monster_count()).collect()
        };
        territories[to_idx].troops = occupying.len() as u32;
        territories[to_idx].body_monster_counts = Some(occupying);
        territories[to_idx].body_names = None;
        push_log(log, format!("{}を占領しました！", to_name));

        let is_ruin = territories[to_idx].ruin.is_some();
        let enemy_type_refs: Vec<&str> = enemy_names.iter().map(|name| name.as_str()).collect();
        let drops = crate::items::calculate_drops(&enemy_type_refs, facility_bonuses.drop_rate);

        if !drops.is_empty() {
            push_log(log, "--- 戦利品 ---".to_string());
            for drop in &drops {
                push_log(log, format!("{}x{} を入手！", drop.item_id, drop.count));
            }
            crate::items::add_items_to_inventory(&mut new_inventory, drops);
        }

        let dropped_cards = calculate_card_drops(&enemy_names, facility_bonuses.drop_rate as f32);
        if !dropped_cards.is_empty() {
            push_log(log, "--- カード入手 ---".to_string());
            for card_id in &dropped_cards {
                if let Some(card) = crate::cards::get_card(*card_id) {
                    push_log(log, format!("カード「{}」を入手！", card.name));
                    new_owned_cards.push(*card_id);
                }
            }
        }

        if is_ruin {
            territories[to_idx].ruin = None;
            push_log(log, "遺跡を攻略しました！".to_string());
        }
    } else if !partial_occupation {
        let remaining_monster_counts: Vec<u32> = surviving_enemies.iter().map(|character| character.effective_monster_count()).collect();
        let remaining_names: Vec<String> = surviving_enemies.iter().map(|character| character.name.clone()).collect();
        territories[to_idx].troops = remaining_monster_counts.len() as u32;
        territories[to_idx].body_monster_counts = Some(remaining_monster_counts);
        territories[to_idx].body_names = if remaining_names.is_empty() { None } else { Some(remaining_names) };
        push_log(log, format!("攻撃失敗。{}の防衛に成功。", to_name));
    }

    let mut new_players = working_players;
    if let Some(player) = new_players.get_mut(DEFAULT_PLAYER_ID) {
        player.inventory = new_inventory.clone();
        player.owned_cards = new_owned_cards.clone();
        if conquered {
            if let Some(ref idxs) = owned_card_indices {
                let xp_gain = 40_u64.saturating_add(facility_bonuses.exp_bonus as u64 / 2);
                while player.card_exp.len() < player.owned_cards.len() {
                    player.card_exp.push(0);
                }
                for &i in idxs {
                    if i < player.card_exp.len() {
                        player.card_exp[i] = player.card_exp[i].saturating_add(xp_gain);
                    }
                }
            }
        }
    }

    build_game_state(
        state,
        state.turn,
        territories,
        log.clone(),
        new_players,
        new_inventory,
        state.facilities.clone(),
        new_owned_cards,
        state.card_skill_levels.clone(),
    )
}

fn apply_start_exploration(
    state: &GameState,
    log: &mut Vec<String>,
    territory_id: &str,
    card_indices: &[usize],
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };
    if get_territory_index(&state.territories, territory_id).is_none() {
        return state.clone();
    }
    let tidx = get_territory_index(&state.territories, territory_id).unwrap();
    if state.territories[tidx].owner_id.as_deref() != Some(DEFAULT_PLAYER_ID) {
        push_log(log, "探索は占領済みの自領地からのみ派遣できます。".to_string());
        return state.clone();
    }
    if is_home_territory(territory_id) {
        push_log(log, "本拠地からは探索を派遣しません。".to_string());
        return state.clone();
    }
    if card_indices.is_empty() {
        push_log(log, "探索に使用するカードを選んでください。".to_string());
        return state.clone();
    }
    let max_slots = player.exploration_level.max(1) as usize;
    if card_indices.len() > max_slots {
        push_log(log, "同時派遣数が探索レベルを超えています。".to_string());
        return state.clone();
    }
    for &i in card_indices {
        if i >= player.owned_cards.len() {
            push_log(log, "無効なカードインデックスです。".to_string());
            return state.clone();
        }
    }
    if player.explorations.len() >= max_slots {
        push_log(log, "これ以上探索を出せません。".to_string());
        return state.clone();
    }
    let now = default_now_ms();
    let mission_id = format!("exp-{}", now);
    player.explorations.push(ExplorationMission {
        mission_id: mission_id.clone(),
        territory_id: territory_id.to_string(),
        started_at: now,
        completes_at: now.saturating_add(3 * 60 * 1000),
        card_indices: card_indices.to_vec(),
    });
    push_log(
        log,
        format!("{} へ探索隊を派遣しました（完了まで約3分）。", territory_id),
    );
    build_game_state(
        state,
        state.turn,
        state.territories.clone(),
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    )
}

fn apply_collect_exploration(state: &GameState, log: &mut Vec<String>, mission_id: &str) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };
    let now = default_now_ms();
    let Some(ix) = player.explorations.iter().position(|m| m.mission_id == mission_id) else {
        push_log(log, "該当する探索がありません。".to_string());
        return state.clone();
    };
    let m = player.explorations[ix].clone();
    if now < m.completes_at {
        push_log(log, "探索はまだ完了していません。".to_string());
        return state.clone();
    }
    player.explorations.remove(ix);
    let score = 10u64.saturating_add(m.card_indices.len() as u64 * 8);
    player.exploration_score = player.exploration_score.saturating_add(score);
    while player.exploration_score >= 100 && player.exploration_level < 6 {
        player.exploration_score -= 100;
        player.exploration_level += 1;
        let lv = player.exploration_level;
        push_log(
            log,
            format!("探索経験が溜まり、探索レベルが {} に上昇！", lv),
        );
    }
    let bonus_food = 30u64.saturating_mul(m.card_indices.len() as u64);
    player.resources.food = player.resources.food.saturating_add(bonus_food);
    push_log(
        log,
        format!("探索完了。スコア+{}、食料+{}", score, bonus_food),
    );
    build_game_state(
        state,
        state.turn,
        state.territories.clone(),
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    )
}

fn apply_donate_alliance(
    state: &GameState,
    log: &mut Vec<String>,
    food: u64,
    wood: u64,
    stone: u64,
    iron: u64,
) -> GameState {
    let total = food.saturating_add(wood).saturating_add(stone).saturating_add(iron);
    if total == 0 {
        return state.clone();
    }
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };
    if player.resources.food < food
        || player.resources.wood < wood
        || player.resources.stone < stone
        || player.resources.iron < iron
    {
        push_log(log, "寄付する資源が足りません。".to_string());
        return state.clone();
    }
    let mut alliances = state.alliances.clone();
    let Some(ai) = alliances
        .iter()
        .position(|a| a.member_ids.iter().any(|m| m == DEFAULT_PLAYER_ID))
    else {
        push_log(log, "同盟に所属していないため寄付できません。".to_string());
        return state.clone();
    };
    player.resources.food -= food;
    player.resources.wood -= wood;
    player.resources.stone -= stone;
    player.resources.iron -= iron;
    alliances[ai].donated_total = alliances[ai].donated_total.saturating_add(total);
    let donated = alliances[ai].donated_total;
    let new_level = ((donated / 2000) as u32).saturating_add(1).min(20).max(1);
    if new_level > alliances[ai].level {
        push_log(
            log,
            format!("同盟への寄付が実を結び、同盟レベルが {} になった！", new_level),
        );
    }
    alliances[ai].level = new_level;
    let mut out = build_game_state(
        state,
        state.turn,
        state.territories.clone(),
        log.clone(),
        players,
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
        state.card_skill_levels.clone(),
    );
    out.alliances = alliances;
    out
}

fn calculate_card_drops(enemy_names: &[String], drop_rate_bonus: f32) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut dropped = Vec::new();

    for name in enemy_names {
        if let Some(card_id) = crate::cards::get_card_id_by_name(name) {
            let Some(card) = crate::cards::get_card(card_id) else {
                continue;
            };
            let base_chance = crate::cards::get_card_drop_chance(card.rarity);
            let actual_chance = base_chance * (1.0 + drop_rate_bonus);
            if rng.gen::<f32>() < actual_chance {
                dropped.push(card_id);
            }
        }
    }

    dropped
}

// ========== フリーマーケット ==========

const BASE_MARKET_FEE_PERCENT: u64 = 10;

fn calculate_market_fee(price: u64, facilities: &[crate::model::BuiltFacility]) -> u64 {
    let bonuses = crate::facilities::calculate_facility_bonuses(facilities);
    let reduction = bonuses.market_fee_reduction.min(BASE_MARKET_FEE_PERCENT as u32) as u64;
    let fee_percent = BASE_MARKET_FEE_PERCENT.saturating_sub(reduction);
    price * fee_percent / 100
}

fn apply_list_on_flea_market(
    state: &GameState,
    log: &mut Vec<String>,
    item: &MarketItemType,
    price: u64,
) -> GameState {
    if price == 0 {
        push_log(log, "価格は1以上に設定してください。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let Some(player) = new_state.players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };

    let item_desc = match item {
        MarketItemType::Card { card_id } => {
            let idx = player.owned_cards.iter().position(|&id| id == *card_id);
            match idx {
                Some(i) => {
                    player.owned_cards.remove(i);
                    let name = crate::cards::get_card(*card_id)
                        .map(|c| c.name.to_string())
                        .unwrap_or_else(|| format!("カード#{}", card_id));
                    name
                }
                None => {
                    push_log(log, "出品するカードを所持していません。".to_string());
                    return state.clone();
                }
            }
        }
        MarketItemType::Item { item_id, count } => {
            let inv_item = player.inventory.iter_mut().find(|i| i.item_id == *item_id);
            match inv_item {
                Some(inv) if inv.count >= *count => {
                    inv.count -= count;
                    if inv.count == 0 {
                        player.inventory.retain(|i| i.item_id != *item_id);
                    }
                    let name = crate::items::item_name(item_id);
                    format!("{}x{}", name, count)
                }
                _ => {
                    push_log(log, "出品するアイテムが足りません。".to_string());
                    return state.clone();
                }
            }
        }
        MarketItemType::Resource { resource_type, amount } => {
            let has_enough = match resource_type.as_str() {
                "food" => player.resources.food >= *amount,
                "wood" => player.resources.wood >= *amount,
                "stone" => player.resources.stone >= *amount,
                "iron" => player.resources.iron >= *amount,
                _ => false,
            };
            if !has_enough || *amount == 0 {
                push_log(log, "出品する資源が足りません。".to_string());
                return state.clone();
            }
            match resource_type.as_str() {
                "food" => player.resources.food -= amount,
                "wood" => player.resources.wood -= amount,
                "stone" => player.resources.stone -= amount,
                "iron" => player.resources.iron -= amount,
                _ => {}
            }
            let type_name = match resource_type.as_str() {
                "food" => "食料",
                "wood" => "木材",
                "stone" => "石材",
                "iron" => "鉄",
                _ => "不明",
            };
            format!("{}x{}", type_name, amount)
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let listing_id = format!("listing_{}_{}", now, new_state.market_listings.len());
    new_state.market_listings.push(MarketListing {
        listing_id,
        seller_id: DEFAULT_PLAYER_ID.to_string(),
        item: item.clone(),
        price,
        listed_at: now,
    });

    new_state.owned_cards = player.owned_cards.clone();
    new_state.inventory = player.inventory.clone();
    push_log(log, format!("フリマに{}を{}Gで出品しました。", item_desc, price));
    new_state.log = log.clone();
    new_state
}

fn apply_buy_from_flea_market(
    state: &GameState,
    log: &mut Vec<String>,
    listing_id: &str,
) -> GameState {
    let listing_idx = state.market_listings.iter().position(|l| l.listing_id == listing_id);
    let Some(idx) = listing_idx else {
        push_log(log, "出品が見つかりません。".to_string());
        return state.clone();
    };
    let listing = &state.market_listings[idx];

    if listing.seller_id == DEFAULT_PLAYER_ID {
        push_log(log, "自分の出品は購入できません。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let Some(buyer) = new_state.players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };

    if buyer.resources.gold < listing.price {
        push_log(log, "ゴールドが足りません。".to_string());
        return state.clone();
    }

    buyer.resources.gold -= listing.price;

    match &listing.item {
        MarketItemType::Card { card_id } => {
            buyer.owned_cards.push(*card_id);
        }
        MarketItemType::Item { item_id, count } => {
            if let Some(inv) = buyer.inventory.iter_mut().find(|i| i.item_id == *item_id) {
                inv.count += count;
            } else {
                buyer.inventory.push(crate::items::InventoryItem {
                    item_id: item_id.clone(),
                    count: *count,
                });
            }
        }
        MarketItemType::Resource { resource_type, amount } => {
            match resource_type.as_str() {
                "food" => buyer.resources.food += amount,
                "wood" => buyer.resources.wood += amount,
                "stone" => buyer.resources.stone += amount,
                "iron" => buyer.resources.iron += amount,
                _ => {}
            }
        }
    }

    let fee = calculate_market_fee(listing.price, &buyer.facilities);
    let seller_id = listing.seller_id.clone();
    let proceeds = listing.price.saturating_sub(fee);

    if let Some(seller) = new_state.players.get_mut(&seller_id) {
        seller.resources.gold += proceeds;
    }

    let item_desc = match &new_state.market_listings[idx].item {
        MarketItemType::Card { card_id } => {
            crate::cards::get_card(*card_id)
                .map(|c| c.name.to_string())
                .unwrap_or_else(|| format!("カード#{}", card_id))
        }
        MarketItemType::Item { item_id, count } => {
            format!("{}x{}", crate::items::item_name(item_id), count)
        }
        MarketItemType::Resource { resource_type, amount } => {
            let name = match resource_type.as_str() {
                "food" => "食料", "wood" => "木材", "stone" => "石材", "iron" => "鉄",
                _ => "不明",
            };
            format!("{}x{}", name, amount)
        }
    };

    new_state.market_listings.remove(idx);

    let buyer_ref = new_state.players.get(DEFAULT_PLAYER_ID).unwrap();
    new_state.owned_cards = buyer_ref.owned_cards.clone();
    new_state.inventory = buyer_ref.inventory.clone();

    push_log(log, format!(
        "フリマで{}を{}Gで購入（手数料{}G）",
        item_desc, listing.price, fee
    ));
    new_state.log = log.clone();
    new_state
}

fn apply_cancel_flea_market_listing(
    state: &GameState,
    log: &mut Vec<String>,
    listing_id: &str,
) -> GameState {
    let listing_idx = state.market_listings.iter().position(|l| l.listing_id == listing_id);
    let Some(idx) = listing_idx else {
        push_log(log, "出品が見つかりません。".to_string());
        return state.clone();
    };

    if state.market_listings[idx].seller_id != DEFAULT_PLAYER_ID {
        push_log(log, "自分の出品のみ取り消せます。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let listing = new_state.market_listings.remove(idx);

    let Some(player) = new_state.players.get_mut(DEFAULT_PLAYER_ID) else {
        return state.clone();
    };

    match &listing.item {
        MarketItemType::Card { card_id } => {
            player.owned_cards.push(*card_id);
        }
        MarketItemType::Item { item_id, count } => {
            if let Some(inv) = player.inventory.iter_mut().find(|i| i.item_id == *item_id) {
                inv.count += count;
            } else {
                player.inventory.push(crate::items::InventoryItem {
                    item_id: item_id.clone(),
                    count: *count,
                });
            }
        }
        MarketItemType::Resource { resource_type, amount } => {
            match resource_type.as_str() {
                "food" => player.resources.food += amount,
                "wood" => player.resources.wood += amount,
                "stone" => player.resources.stone += amount,
                "iron" => player.resources.iron += amount,
                _ => {}
            }
        }
    }

    new_state.owned_cards = player.owned_cards.clone();
    new_state.inventory = player.inventory.clone();
    push_log(log, "出品を取り消しました。".to_string());
    new_state.log = log.clone();
    new_state
}
