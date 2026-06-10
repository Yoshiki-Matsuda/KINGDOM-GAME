use super::*;

pub(super) fn apply_end_turn_action(state: &GameState, log: &mut Vec<String>) -> GameState {
    push_log(log, format!("--- ターン {} 終了 ---", state.turn));
    let mut players = state.players.clone();
    for player in players.values_mut() {
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
    build_game_state(state, state.turn + 1, state.territories.clone(), log.clone(), players)
}

pub(super) fn apply_deploy_action(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
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
    let allied_player_ids = state
        .players
        .get(actor_player_id)
        .map(|p| p.allied_player_ids.as_slice())
        .unwrap_or(&[] as &[String]);
    if !can_receive_reinforcement(&territories, actor_player_id, allied_player_ids, territory_id) {
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

    build_game_state(state, state.turn, territories, log.clone(), state.players.clone())
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

/// ローカル開発用: 攻撃側を敵最強体の10倍相当まで引き上げ（プレイヤー上限9999は適用しない）
fn apply_dev_auto_win_boost(our_chars: &mut [CombatCharacter], enemy_chars: &[CombatCharacter]) {
    let max_enemy = enemy_chars
        .iter()
        .map(|c| c.current_monster_count.max(c.base_monster_count as f32))
        .fold(0.0f32, f32::max)
        .max(1.0);
    let target_mc = (max_enemy * 10.0).min(999_999.0);
    for character in our_chars.iter_mut() {
        if !character.is_alive {
            continue;
        }
        character.current_monster_count = character.current_monster_count.max(target_mc);
        character.damage_multiplier *= 10.0;
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

/// 種族研究所Lv ボーナス: 該当種族の魔獣に攻防+1%/Lv（最大+10%）
/// KC仕様: 獣族生態研究所・亜人族行動研究所・不死族解析研究所・精霊族調査研究所・
///         巨人族監視研究所・魔族防衛研究所・龍族探索研究所 等
fn apply_race_lab_bonus(
    chars: &mut [CombatCharacter],
    facilities: &[crate::model::BuiltFacility],
    log: &mut Vec<String>,
) {
    use crate::cards::Race;
    let now = default_now_ms();
    let mut lab_levels: std::collections::HashMap<Race, u32> = std::collections::HashMap::new();
    for f in facilities {
        if let Some(complete_at) = f.build_complete_at {
            if complete_at > now {
                continue;
            }
        }
        let race = match f.facility_id.as_str() {
            "beast_lab" => Some(Race::Beast),
            "demihuman_lab" => Some(Race::Demihuman),
            "demon_lab" => Some(Race::Demon),
            "dragon_lab" => Some(Race::Dragon),
            "giant_lab" => Some(Race::Giant),
            "spirit_lab" => Some(Race::Spirit),
            "undead_lab" => Some(Race::Undead),
            _ => None,
        };
        if let Some(r) = race {
            let lv = f.level as u32;
            let entry = lab_levels.entry(r).or_insert(0);
            if lv > *entry {
                *entry = lv;
            }
        }
    }
    if lab_levels.is_empty() {
        return;
    }
    for c in chars.iter_mut() {
        if let Some(r) = c.race {
            if let Some(lv) = lab_levels.get(&r).copied() {
                let pct = (lv.min(10) as f32) / 100.0;
                let atk_bonus = (c.attack as f32 * pct).ceil() as u32;
                let def_bonus = (c.defense as f32 * pct).ceil() as u32;
                c.attack += atk_bonus;
                c.defense += def_bonus;
            }
        }
    }
    for (race, lv) in &lab_levels {
        let race_name = match race {
            Race::Beast => "獣族",
            Race::Demihuman => "亜人族",
            Race::Demon => "魔族",
            Race::Dragon => "龍族",
            Race::Giant => "巨人族",
            Race::Spirit => "精霊族",
            Race::Undead => "不死族",
        };
        let pct = lv.min(&10);
        push_log(
            log,
            format!("◆ {}研究所Lv{}ボーナス（攻撃・防御+{}%）", race_name, lv, pct),
        );
    }
}

/// 戦神の祠・守護神の祠の攻防ボーナスを戦闘キャラに適用
/// KC仕様: 拠点内の魔獣に攻/防+2%/Lv（戦神→攻撃、守護神→防御）
fn apply_shrine_bonus(
    chars: &mut [CombatCharacter],
    bonuses: &crate::facilities::FacilityBonuses,
    log: &mut Vec<String>,
) {
    if bonuses.attack_bonus == 0 && bonuses.defense_bonus == 0 {
        return;
    }
    let atk_pct = bonuses.attack_bonus as f32 / 100.0;
    let def_pct = bonuses.defense_bonus as f32 / 100.0;
    for c in chars.iter_mut() {
        if atk_pct > 0.0 {
            let b = (c.attack as f32 * atk_pct).ceil() as u32;
            c.attack += b;
        }
        if def_pct > 0.0 {
            let b = (c.defense as f32 * def_pct).ceil() as u32;
            c.defense += b;
        }
    }
    if bonuses.attack_bonus > 0 {
        push_log(
            log,
            format!("◆ 戦神の祠ボーナス（攻撃+{}%）", bonuses.attack_bonus),
        );
    }
    if bonuses.defense_bonus > 0 {
        push_log(
            log,
            format!("◆ 守護神の祠ボーナス（防御+{}%）", bonuses.defense_bonus),
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
        || mods.aoe_percent_damage > 1e-4
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
        push_log(log, format!("+{:.0} 割合ダメージ", percent_dmg));
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

/// 占領成功時、出撃した魔獣スロットへ経験値を付与しレベルアップ処理を行う
/// （KC仕様: スタミナ50未満はXPなし、満タン出撃はボーナス）
fn award_conquest_xp(
    player: &mut crate::model::PlayerData,
    idxs: &[usize],
    facility_bonuses: &crate::facilities::FacilityBonuses,
    log: &mut Vec<String>,
) {
    let base_xp = 40_u64.saturating_add(facility_bonuses.exp_bonus as u64 / 2);
    while player.card_exp.len() < player.owned_cards.len() {
        player.card_exp.push(0);
    }
    while player.card_levels.len() < player.owned_cards.len() {
        player.card_levels.push(1);
    }
    while player.card_status_points.len() < player.owned_cards.len() {
        player.card_status_points.push(0);
    }
    for &i in idxs {
        if i >= player.card_exp.len() {
            continue;
        }
        // KC仕様: スタミナ200以下ではXPなし、MAX出撃(120=満タン)ならボーナス
        let stam_before = player
            .card_stamina
            .get(i)
            .copied()
            .unwrap_or(120)
            .saturating_add(crate::model_actions::STAMINA_ATTACK_FOR_XP);
        let xp_gain = if stam_before < 50 {
            0
        } else if stam_before >= 120 {
            base_xp.saturating_mul(6) / 5
        } else {
            base_xp
        };
        if xp_gain == 0 {
            continue;
        }
        player.card_exp[i] = player.card_exp[i].saturating_add(xp_gain);
        let name = crate::cards::get_card(player.owned_cards[i])
            .map(|c| c.name.to_string())
            .unwrap_or_else(|| format!("魔獣#{}", player.owned_cards[i]));
        let awakened = *player.card_levels.get(i).unwrap_or(&1) > 99;
        let mut lvl = player.card_levels[i];
        let mut exp = player.card_exp[i];
        let mut sp = player.card_status_points[i];
        crate::model::process_level_up(&mut lvl, &mut exp, &mut sp, awakened, &name, log);
        player.card_levels[i] = lvl;
        player.card_exp[i] = exp;
        player.card_status_points[i] = sp;
    }
}

/// 敵ユニット名から CombatCharacter を生成（カードマスタがあればその性能、なければ既定値）
fn make_enemy_char(index: usize, name: &str, monster_count: u32) -> CombatCharacter {
    let card_name = name.trim_end_matches(|c: char| c == 'A' || c == 'B' || c == 'C');
    if let Some(card) = crate::cards::get_card_by_name(card_name) {
        let mut ch = CombatCharacter::with_stats(
            index + 100,
            name.to_string(),
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
        CombatCharacter::new(index + 100, name.to_string(), monster_count, 5, SkillData::default())
    }
}

/// 攻撃側ユニットの CombatCharacter 群を生成（施設ボーナス・強化★補正込み）
#[allow(clippy::too_many_arguments)]
fn build_attacker_chars(
    our_names: &[String],
    our_stats: &[CardStats],
    our_body_monster_counts: &[u32],
    our_speeds: &[u32],
    our_skills: &[SkillData],
    enhanced_flags: &[bool],
    facility_bonuses: &crate::facilities::FacilityBonuses,
) -> Vec<CombatCharacter> {
    our_names
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
                crate::facilities::apply_monster_bonus(base_monster_count, facility_bonuses);
            let boosted_speed = base_speed + facility_bonuses.speed_bonus;

            // KC: 強化魔獣 ★ はステータス +10%
            let enhanced = *enhanced_flags.get(index).unwrap_or(&false);
            let mul = |v: u32| if enhanced { ((v as f32) * 1.10).round() as u32 } else { v };
            let attack = mul(attack);
            let intelligence = mul(intelligence);
            let defense = mul(defense);
            let magic_defense = mul(magic_defense);
            let boosted_speed = mul(boosted_speed);
            let boosted_monster_count = mul(boosted_monster_count);

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
            if enhanced {
                ch.name = format!("★{}", ch.name);
            }
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
        .collect()
}

/// 攻撃ダメージ適用後の結果処理（処刑・全体攻撃・シールド・撃破・反射・反撃）。
///
/// `teams[0]` は行動側チーム（actor_idx の所属）、`teams[1]` は相手チーム。
/// 魅了・混乱による同士討ちでは `target_team == 0` になる。
/// 死亡スキル（復活等）の判定はプレイヤー側キャラが倒れたときのみ行う（KC準拠）。
#[allow(clippy::too_many_arguments)]
fn resolve_attack_outcome(
    teams: &mut [&mut Vec<CombatCharacter>; 2],
    actor_idx: usize,
    target_team: usize,
    target_idx: usize,
    actor_is_player: bool,
    target_team_is_player: bool,
    atk_name: &str,
    def_name: &str,
    net_damage: f32,
    attack_mods: &crate::skills::AttackModifiers,
    log: &mut Vec<String>,
) {
    // 処刑（残存率が閾値以下なら即死、死亡スキル判定なし）
    let hp_ratio = teams[target_team][target_idx].current_monster_count
        / teams[target_team][target_idx].base_monster_count as f32;
    if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
        teams[target_team][target_idx].is_alive = false;
        teams[target_team][target_idx].current_monster_count = 0.0;
        push_log(log, format!("処刑発動！{}を即死させた！", def_name));
        return;
    }

    // 全体攻撃（固定 + 割合）
    if attack_mods.aoe_damage > 0.0 || attack_mods.aoe_percent_damage > 0.0 {
        let label = if target_team_is_player {
            if actor_is_player { "味方全員" } else { "プレイヤー側全員" }
        } else {
            "敵全員"
        };
        for member in teams[target_team].iter_mut() {
            if !member.is_alive {
                continue;
            }
            let splash = attack_mods.aoe_damage
                + member.current_monster_count * attack_mods.aoe_percent_damage;
            if splash <= 0.0 {
                continue;
            }
            member.current_monster_count -= splash;
            if member.current_monster_count <= 0.0 {
                member.is_alive = false;
                if !(target_team_is_player && check_death_skills(member, log)) {
                    push_log(log, format!("{}が全体攻撃で撃破されました。", member.name));
                }
            }
        }
        push_log(
            log,
            format!(
                "全体攻撃で{}にダメージ！（固定{:.0}{}）",
                label,
                attack_mods.aoe_damage,
                if attack_mods.aoe_percent_damage > 0.0 {
                    format!(" + 各HP{:.1}%", attack_mods.aoe_percent_damage * 100.0)
                } else {
                    String::new()
                }
            ),
        );
    }

    // シールド吸収後に本体ダメージ
    let damage_after_shield = teams[target_team][target_idx].absorb_damage_with_shield(net_damage);
    if damage_after_shield > 0.0 {
        teams[target_team][target_idx].current_monster_count -= damage_after_shield;
    }

    if teams[target_team][target_idx].current_monster_count <= 0.0 {
        teams[target_team][target_idx].is_alive = false;
        let death_logged =
            target_team_is_player && check_death_skills(&mut teams[target_team][target_idx], log);
        if !death_logged {
            if !actor_is_player && target_team_is_player {
                push_log(log, format!("{}が撃破されました。", def_name));
            } else {
                push_log(log, format!("{}が{}を撃破しました。", atk_name, def_name));
            }
        }

        if attack_mods.monster_steal > 0.0 {
            teams[0][actor_idx].current_monster_count += attack_mods.monster_steal;
            push_log(log, format!("{}が {:.0} 魔獣数を奪取！", atk_name, attack_mods.monster_steal));
        }
        if attack_mods.absorb_rate > 0.0 {
            let absorb = damage_after_shield * attack_mods.absorb_rate;
            teams[0][actor_idx].current_monster_count += absorb;
            push_log(log, format!("{}が {:.0} 魔獣数を吸収！", atk_name, absorb));
        }
        if attack_mods.extra_attacks > 0 {
            teams[0][actor_idx].extra_attacks += attack_mods.extra_attacks;
            push_log(log, format!("{}が追加攻撃権を得た！", atk_name));
        }
    } else if damage_after_shield <= 0.0 {
        push_log(log, format!("{}のシールドがダメージを吸収！", def_name));
    } else {
        // 反射
        let reflect_rate = teams[target_team][target_idx].get_reflect_rate();
        if reflect_rate > 0.0 {
            let reflect_damage = net_damage * reflect_rate;
            teams[0][actor_idx].current_monster_count -= reflect_damage;
            push_log(log, format!("{}の反射で {:.0} ダメージ！", def_name, reflect_damage));
            if teams[0][actor_idx].current_monster_count <= 0.0 {
                teams[0][actor_idx].is_alive = false;
                if !(actor_is_player && check_death_skills(&mut teams[0][actor_idx], log)) {
                    push_log(log, format!("{}が反射で撃破されました。", atk_name));
                }
            }
        }
        // 反撃
        let counter_rate = teams[target_team][target_idx].get_counter_rate();
        if counter_rate > 0.0
            && rand::random::<f32>() < counter_rate
            && teams[0][actor_idx].is_alive
        {
            let c_atk = teams[target_team][target_idx]
                .attack
                .max(teams[target_team][target_idx].intelligence) as f32
                * teams[target_team][target_idx].attack_buff_multiplier();
            let c_def = teams[0][actor_idx]
                .defense
                .max(teams[0][actor_idx].magic_defense) as f32
                * teams[0][actor_idx].defense_buff_multiplier();
            let c_mc = teams[target_team][target_idx].current_monster_count;
            let c_ratio = (c_atk / c_def.max(1.0)).clamp(0.3, 1.1);
            let counter_raw = c_ratio
                * c_mc
                * 0.5
                * teams[target_team][target_idx].outgoing_damage_multiplier();
            let c_min = kc_minimum_damage(c_mc) * 0.5;
            let c_max = c_mc * 0.55;
            let (cl, ch) = if c_min <= c_max {
                (c_min, c_max)
            } else {
                (c_max, c_max)
            };
            let counter_dmg = counter_raw.clamp(cl, ch);
            teams[0][actor_idx].current_monster_count -= counter_dmg;
            push_log(log, format!("{}の反撃！ {:.0} ダメージ！", def_name, counter_dmg));
            if teams[0][actor_idx].current_monster_count <= 0.0 {
                teams[0][actor_idx].is_alive = false;
                if !(actor_is_player && check_death_skills(&mut teams[0][actor_idx], log)) {
                    push_log(log, format!("{}が反撃で撃破されました。", atk_name));
                }
            }
        }
    }
}

/// 戦闘ターン中の1キャラ分の行動を処理する。
/// 対象選択（魅了・混乱の同士討ち含む）→攻撃スキル→回避/無敵→ダメージ→反射・反撃→追撃効果。
fn perform_actor_turn(
    attacker_team: &mut Vec<CombatCharacter>,
    opponent_team: &mut Vec<CombatCharacter>,
    actor_idx: usize,
    actor_is_player: bool,
    log: &mut Vec<String>,
) {
    if !attacker_team[actor_idx].is_alive {
        return;
    }

    if attacker_team[actor_idx].is_disabled() {
        push_log(log, format!("{}は行動不能！", attacker_team[actor_idx].name));
        return;
    }

    // 魅了・混乱時は自陣営を狙う
    let target_own_team = {
        let ch = &attacker_team[actor_idx];
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
    let target_team_is_player = if target_own_team { actor_is_player } else { !actor_is_player };

    let target_idx = if target_own_team {
        match find_target_excluding(
            attacker_team[actor_idx].range,
            attacker_team.as_slice(),
            Some(actor_idx),
        ) {
            Some(i) => i,
            None => return,
        }
    } else {
        match find_target(attacker_team[actor_idx].range, opponent_team.as_slice()) {
            Some(i) => i,
            None => return,
        }
    };

    let attack_mods = if attacker_team[actor_idx].is_silenced() {
        push_log(log, format!("{}は沈黙中でスキル使用不可！", attacker_team[actor_idx].name));
        crate::skills::AttackModifiers::new()
    } else {
        apply_attack_skills(&mut attacker_team[actor_idx], actor_is_player, log)
    };

    // ダメージを伴わないスキル発動はバフ・デバフ付与のみで行動終了
    if attack_mods.skill_activated && !attack_mods_has_skill_damage(&attack_mods) {
        if target_own_team {
            apply_on_attack_skill_followup_one_team(&attack_mods, attacker_team, actor_idx, target_idx, log);
        } else {
            apply_on_attack_skill_followup(&attack_mods, attacker_team, opponent_team, actor_idx, target_idx, log);
        }
        return;
    }

    let atk_name = attacker_team[actor_idx].name.clone();
    let def_name = if target_own_team {
        attacker_team[target_idx].name.clone()
    } else {
        opponent_team[target_idx].name.clone()
    };

    let evasion_rate = if target_own_team {
        attacker_team[target_idx].get_evasion_rate()
    } else {
        opponent_team[target_idx].get_evasion_rate()
    };
    if evasion_rate > 0.0 && rand::random::<f32>() < evasion_rate {
        push_log(log, format!("{}の攻撃を{}が回避！", atk_name, def_name));
        return;
    }

    let blocked_by_invincible = if target_own_team {
        attacker_team[target_idx].consume_invincible()
    } else {
        opponent_team[target_idx].consume_invincible()
    };
    if blocked_by_invincible {
        push_log(log, format!("{}は無敵で攻撃を無効化！", def_name));
        return;
    }

    let net_damage = {
        let a = &attacker_team[actor_idx];
        let d = if target_own_team {
            &attacker_team[target_idx]
        } else {
            &opponent_team[target_idx]
        };
        compute_net_attack_damage(a, d, &attack_mods, log)
    };
    push_log(
        log,
        format!(
            "{} {}が{}に攻撃！（{:.0} ダメージ）",
            crate::skills::side_label(actor_is_player),
            atk_name,
            def_name,
            net_damage
        ),
    );

    let target_team = if target_own_team { 0 } else { 1 };
    resolve_attack_outcome(
        &mut [&mut *attacker_team, &mut *opponent_team],
        actor_idx,
        target_team,
        target_idx,
        actor_is_player,
        target_team_is_player,
        &atk_name,
        &def_name,
        net_damage,
        &attack_mods,
        log,
    );

    if target_own_team {
        apply_on_attack_skill_followup_one_team(&attack_mods, attacker_team, actor_idx, target_idx, log);
    } else {
        apply_on_attack_skill_followup(&attack_mods, attacker_team, opponent_team, actor_idx, target_idx, log);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_attack_action(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
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
    if territories[from_idx].owner_id.as_deref() != Some(actor_player_id) {
        push_log(log, format!("出撃元({})を所有していません。", from_territory_id));
        return state.clone();
    }
    // 本拠編成（owned_card_indices）の遠征: from は隣接ルート用のみ。編成体数は本拠で判定する
    let expedition_from_home = owned_card_indices.is_some();
    if expedition_from_home {
        let Some(player) = state.players.get(actor_player_id) else {
            return state.clone();
        };
        let home_id = player.home_territory_id.as_str();
        let Some(home_idx) = get_territory_index(&territories, home_id) else {
            push_log(log, "本拠地が見つかりません。".to_string());
            return state.clone();
        };
        if count == 0 || territories[home_idx].troops < count {
            push_log(
                log,
                format!(
                    "本拠({})の編成体数が足りません（必要{}体, 現在{}体）。",
                    home_id, count, territories[home_idx].troops
                ),
            );
            return state.clone();
        }
    } else if territories[from_idx].troops < count || count == 0 {
        push_log(
            log,
            format!(
                "出撃元({})の駐留体数が足りません（必要{}体, 現在{}体）。",
                from_territory_id, count, territories[from_idx].troops
            ),
        );
        return state.clone();
    }
    let base_owners = attack_base_owner_ids(state, actor_player_id);
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

    let player_facilities = state
        .players
        .get(actor_player_id)
        .map(|p| p.facilities.as_slice())
        .unwrap_or(&[] as &[crate::model::BuiltFacility]);
    let player_card_enhanced = state
        .players
        .get(actor_player_id)
        .map(|p| p.card_enhanced.as_slice())
        .unwrap_or(&[] as &[bool]);
    let facility_bonuses = crate::facilities::calculate_facility_bonuses(player_facilities);
    let cost_cap = state
        .players
        .get(actor_player_id)
        .map(|p| p.unit_cost_cap)
        .unwrap_or(4.0)
        + facility_bonuses.unit_cost_cap_bonus;
    // KC: ★強化魔獣はユニットコスト -25%
    let total_cost: f32 = if our_stats.len() == count as usize {
        our_stats
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let enhanced = owned_card_indices
                    .as_ref()
                    .and_then(|oci| oci.get(i).copied())
                    .and_then(|idx| player_card_enhanced.get(idx).copied())
                    .unwrap_or(false);
                if enhanced { s.cost * 0.75 } else { s.cost }
            })
            .sum()
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

    // KC準拠: 攻撃1回でスタミナ消費
    // （XP計算時は消費前の値と比較するため公開する）

    let mut working_players = state.players.clone();

    if let Some(ref oci) = owned_card_indices {
        if oci.len() != count as usize {
            return state.clone();
        }
        let mut seen_slots = HashSet::new();
        for &i in oci {
            if !seen_slots.insert(i) {
                push_log(log, "同一体スロットを複数回指定できません。".to_string());
                return state.clone();
            }
        }
        let Some(player) = working_players.get(actor_player_id) else {
            return state.clone();
        };
        for &i in oci {
            if i >= player.owned_cards.len() {
                push_log(log, "無効な魔獣スロットです。".to_string());
                return state.clone();
            }
        }
        let mut seen_card_ids = HashSet::new();
        for &i in oci {
            let card_id = player.owned_cards[i];
            if !seen_card_ids.insert(card_id) {
                push_log(log, "同一種の魔獣を複数配置できません。".to_string());
                return state.clone();
            }
        }
        for &i in oci {
            let st = player.card_stamina.get(i).copied().unwrap_or(120);
            if st < STAMINA_ATTACK_FOR_XP {
                push_log(log, "スタミナが足りない魔獣が含まれています。".to_string());
                return state.clone();
            }
        }
    }
    if let Some(ref oci) = owned_card_indices {
        if let Some(player) = working_players.get_mut(actor_player_id) {
            while player.card_stamina.len() < player.owned_cards.len() {
                player.card_stamina.push(120);
            }
            for &i in oci {
                player.card_stamina[i] = player.card_stamina[i].saturating_sub(STAMINA_ATTACK_FOR_XP);
            }
        }
    }

    if owned_card_indices.is_none() {
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
    }

    let from_name = territory_name(&territories, from_territory_id).to_string();
    let to_name = territory_name(&territories, to_territory_id).to_string();
    let defender_territory_id = territories[to_idx].id.clone();

    // 強化魔獣(★)判定のため、このユニットの各スロットに対応する owned_cards インデックスを保持
    let enhanced_flags: Vec<bool> = match owned_card_indices.as_ref() {
        Some(oci) => oci
            .iter()
            .map(|&i| *player_card_enhanced.get(i).unwrap_or(&false))
            .collect(),
        None => vec![false; our_names.len()],
    };

    let mut our_skills_with_levels = our_skills;
    if let Some(ref oci) = owned_card_indices {
        if let Some(player) = state.players.get(actor_player_id) {
            for (j, skill) in our_skills_with_levels.iter_mut().enumerate() {
                if let Some(&card_idx) = oci.get(j) {
                    crate::skills::apply_owned_card_skill_levels(
                        skill,
                        card_idx,
                        &player.card_skill_levels,
                    );
                }
            }
        }
    }

    let mut our_chars: Vec<CombatCharacter> = build_attacker_chars(
        &our_names,
        &our_stats,
        &our_body_monster_counts,
        &our_speeds,
        &our_skills_with_levels,
        &enhanced_flags,
        &facility_bonuses,
    );
    assign_positions(&mut our_chars);

    let (_defender_troops, enemy_monster_counts, enemy_names) =
        resolve_territory_defenders(&territories[to_idx]);

    let mut enemy_chars: Vec<CombatCharacter> = enemy_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let monster_count = *enemy_monster_counts.get(index).unwrap_or(&1);
            make_enemy_char(index, name, monster_count)
        })
        .collect();
    assign_positions(&mut enemy_chars);

    let attacker_label = attack_unit_name.as_deref().unwrap_or(from_name.as_str());
    let coords_str = parse_territory_coords(to_territory_id)
        .map(|(col, row)| format!("<{},{}>", col, row))
        .unwrap_or_default();
    push_log(
        log,
        format!(
            "[p:{actor_player_id}]【{}{}侵攻戦】{}が{}へ侵攻開始",
            to_name, coords_str, attacker_label, to_name
        ),
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
        let wave_seed = format!("{defender_territory_id}:wave{wave}");
        let (_cnt, monster_counts, names) =
            generate_neutral_enemies_for_territory(level, &wave_seed);
        enemy_chars.clear();
        let enemy_names_wave: Vec<String> = names;
        let enemy_monster_counts_wave: Vec<u32> = monster_counts;
        for (i, name) in enemy_names_wave.iter().enumerate() {
            let monster_count = *enemy_monster_counts_wave.get(i).unwrap_or(&1);
            enemy_chars.push(make_enemy_char(i, name, monster_count));
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

    if dev_auto_win {
        apply_dev_auto_win_boost(&mut our_chars, &enemy_chars);
    }

    push_log(log, "--- スタートアップフェーズ ---".to_string());
    if wave == 1 {
        apply_race_bonus(&mut our_chars, log);
        apply_race_lab_bonus(&mut our_chars, player_facilities, log);
        apply_shrine_bonus(&mut our_chars, &facility_bonuses, log);
        apply_race_bonus(&mut enemy_chars, log);
    }
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
                perform_actor_turn(&mut our_chars, &mut enemy_chars, actor_idx, true, log);
            } else {
                perform_actor_turn(&mut enemy_chars, &mut our_chars, actor_idx, false, log);
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

    let (mut new_inventory, mut new_owned_cards) = working_players
        .get(actor_player_id)
        .map(|player| (player.inventory.clone(), player.owned_cards.clone()))
        .unwrap_or_else(|| (Vec::new(), Vec::new()));

    let prev_owner_id = territories[to_idx].owner_id.clone();
    let was_base = territories[to_idx].is_base;
    if conquered {
        territories[to_idx].owner_id = Some(actor_player_id.to_string());
        territories[to_idx].max_durability = 0;
        territories[to_idx].durability = 0;
        territories[to_idx].tower_level = 0;
        if owned_card_indices.is_some() {
            // 本拠遠征: 帰還後は本拠の魔獣数に反映。新占領地は空き（別途派兵で駐留）
            territories[to_idx].troops = 0;
            territories[to_idx].body_monster_counts = Some(vec![]);
            territories[to_idx].body_names = None;
        } else {
            let occupying: Vec<u32> = if surviving_allies.is_empty() {
                vec![1u32]
            } else {
                surviving_allies
                    .iter()
                    .map(|character| character.effective_monster_count())
                    .collect()
            };
            territories[to_idx].troops = occupying.len() as u32;
            territories[to_idx].body_monster_counts = Some(occupying);
            territories[to_idx].body_names = None;
        }
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
            push_log(log, "--- 魔獣入手 ---".to_string());
            for card_id in &dropped_cards {
                if let Some(card) = crate::cards::get_card(*card_id) {
                    push_log(log, format!("魔獣「{}」を入手！", card.name));
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
    if let Some(player) = new_players.get_mut(actor_player_id) {
        player.inventory = new_inventory.clone();
        player.owned_cards = new_owned_cards.clone();
        ensure_card_monster_counts(player);
        if let Some(ref oci) = owned_card_indices {
            if oci.len() == our_chars.len() {
                for (j, &card_idx) in oci.iter().enumerate() {
                    if card_idx < player.card_monster_counts.len() {
                        let surv = our_chars
                            .get(j)
                            .map(|c| c.effective_monster_count())
                            .unwrap_or(crate::model::MIN_MONSTER_COUNT_PER_CARD_SLOT);
                        player.card_monster_counts[card_idx] = surv.clamp(
                            crate::model::MIN_MONSTER_COUNT_PER_CARD_SLOT,
                            crate::model::MAX_MONSTER_COUNT_PER_CARD_SLOT,
                        );
                    }
                }
            }
        }
        sync_home_territory_body_counts_from_player(&mut territories, player);
        if conquered {
            if let Some(ref idxs) = owned_card_indices {
                award_conquest_xp(player, idxs, &facility_bonuses, log);
            }
        }
    }

    // KC仕様: 本拠地陥落 → 対象プレイヤーの所属同盟が、攻撃側の同盟の配下同盟となる
    let mut alliances = state.alliances.clone();
    if conquered && was_base {
        if let Some(prev) = prev_owner_id.as_deref() {
            subjugate_alliance_of(prev, actor_player_id, &mut alliances, log);
        }
    }

    let mut out = build_game_state(state, state.turn, territories, log.clone(), new_players);
    out.alliances = alliances;
    out
}
