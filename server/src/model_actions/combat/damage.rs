use super::*;

/// KC準拠の最低ダメージ（魔獣数に応じた段階式、上限は max_dmg と整合するよう cap）
pub(crate) fn kc_minimum_damage(mc: f32) -> f32 {
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
pub(crate) fn find_target(attacker_range: u8, enemies: &[CombatCharacter]) -> Option<usize> {
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
pub(crate) fn find_target_excluding(
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

pub(crate) fn attack_mods_has_skill_damage(mods: &crate::skills::AttackModifiers) -> bool {
    (mods.damage_multiplier - 1.0).abs() > 1e-4
        || mods.damage_add > 1e-4
        || mods.true_damage > 1e-4
        || mods.percent_damage > 1e-4
        || mods.aoe_damage > 1e-4
        || mods.aoe_percent_damage > 1e-4
        || mods.execute_threshold > 1e-4
}

/// 攻撃時スキルによる付与効果（状態異常・自己バフ・味方列・回復）
pub(crate) fn apply_on_attack_skill_followup(
    mods: &crate::skills::AttackModifiers,
    attackers: &mut [CombatCharacter],
    defenders: &mut [CombatCharacter],
    attacker_idx: usize,
    defender_idx: usize,
    log: &mut Vec<GameEvent>,
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
pub(crate) fn apply_on_attack_skill_followup_one_team(
    mods: &crate::skills::AttackModifiers,
    team: &mut [CombatCharacter],
    attacker_idx: usize,
    defender_idx: usize,
    log: &mut Vec<GameEvent>,
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

pub(crate) fn compute_physical_attack_damage(
    attacker: &CombatCharacter,
    defender: &CombatCharacter,
) -> f32 {
    let atk_stat = attacker.attack as f32 * attacker.attack_buff_multiplier();
    let def_stat = defender.defense as f32
        * (1.0 - defender.damage_reduction)
        * defender.defense_buff_multiplier();
    let mc = attacker.current_monster_count;
    let ratio = (atk_stat / def_stat.max(1.0)).clamp(0.3, 1.1);
    ratio * mc * attacker.damage_multiplier * attacker.outgoing_damage_multiplier()
}

pub(crate) fn compute_skill_attack_damage(
    attacker: &CombatCharacter,
    defender: &CombatCharacter,
    attack_mods: &crate::skills::AttackModifiers,
    log: &mut Vec<GameEvent>,
) -> f32 {
    let int_stat = attacker.intelligence as f32 * attacker.attack_buff_multiplier();
    let def_stat = if attack_mods.ignore_defense {
        0.0
    } else {
        defender.magic_defense as f32
            * (1.0 - defender.damage_reduction)
            * defender.defense_buff_multiplier()
    };
    let mc = attacker.current_monster_count;
    let ratio = (int_stat / def_stat.max(1.0)).clamp(0.3, 1.1);
    let mut raw_damage = ratio * mc * attacker.outgoing_damage_multiplier();
    raw_damage = raw_damage * attack_mods.damage_multiplier + attack_mods.damage_add;
    raw_damage += attack_mods.true_damage;

    if attack_mods.percent_damage > 0.0 {
        let percent_dmg = defender.current_monster_count * attack_mods.percent_damage;
        raw_damage += percent_dmg;
        push_skill_effect_event(log, &format!("+{:.0} 割合ダメージ", percent_dmg));
    }
    raw_damage
}

pub(crate) fn compute_net_attack_damage(
    attacker: &CombatCharacter,
    defender: &CombatCharacter,
    attack_mods: &crate::skills::AttackModifiers,
    log: &mut Vec<GameEvent>,
) -> f32 {
    let mut raw_damage = compute_physical_attack_damage(attacker, defender);
    if attack_mods_has_skill_damage(attack_mods) {
        raw_damage += compute_skill_attack_damage(attacker, defender, attack_mods, log);
    }

    let vulnerability = defender.get_vulnerability();
    let mark_damage = defender.get_mark_damage();
    raw_damage = raw_damage * (1.0 + vulnerability) + mark_damage;
    raw_damage *= race_matchup_damage_multiplier(attacker.race, defender.race);

    let mc = attacker.current_monster_count;
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
pub(crate) fn award_conquest_xp(
    player: &mut crate::model::PlayerData,
    idxs: &[usize],
    facility_bonuses: &crate::facilities::FacilityBonuses,
    log: &mut Vec<GameEvent>,
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
        player.card_exp[i] = player.card_exp[i].saturating_add(base_xp);
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
pub(crate) fn make_enemy_char(index: usize, name: &str, monster_count: u32) -> CombatCharacter {
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
pub(crate) fn build_attacker_chars(
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
            let base_monster_count = our_body_monster_counts
                .get(index)
                .copied()
                .filter(|&c| c > 0)
                .or_else(|| {
                    if stats.monster_count > 0 {
                        Some(stats.monster_count)
                    } else {
                        None
                    }
                })
                .unwrap_or(1);
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
            let boosted_monster_count = mul(boosted_monster_count)
                .min(crate::model::MAX_MONSTER_COUNT_PER_CARD_SLOT);

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
