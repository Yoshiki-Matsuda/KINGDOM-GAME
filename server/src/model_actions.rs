use rand::Rng;

use crate::model::{
    build_game_state,
    can_receive_reinforcement,
    get_territory_index,
    home_territory_id,
    is_attackable_target,
    is_home_territory,
    parse_territory_coords,
    push_log,
    territory_name,
    Action,
    CardStats,
    GameState,
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
    match action {
        Action::EndTurn => apply_end_turn_action(state, &mut log),
        Action::Deploy {
            territory_id,
            count,
            energy_per_body,
            body_names,
        } => apply_deploy_action(state, &mut log, territory_id, *count, energy_per_body, body_names),
        Action::Attack {
            from_territory_id,
            to_territory_id,
            count,
            energy_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
        } => apply_attack_action(
            state,
            &mut log,
            from_territory_id,
            to_territory_id,
            *count,
            energy_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            dev_auto_win,
        ),
    }
}

fn apply_end_turn_action(state: &GameState, log: &mut Vec<String>) -> GameState {
    push_log(log, format!("--- ターン {} 終了 ---", state.turn));
    build_game_state(
        state,
        state.turn + 1,
        state.territories.clone(),
        log.clone(),
        state.players.clone(),
        state.inventory.clone(),
        state.facilities.clone(),
        state.owned_cards.clone(),
    )
}

fn apply_deploy_action(
    state: &GameState,
    log: &mut Vec<String>,
    territory_id: &str,
    count: u32,
    energy_per_body: &Option<Vec<u32>>,
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

    let reinforcement_energies: Vec<u32> = energy_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![1u32; count as usize]);
    if let Some(ref mut values) = territories[idx].body_energies {
        values.extend(reinforcement_energies.iter());
    } else {
        let existing = territories[idx].troops.saturating_sub(count) as usize;
        let mut next_energies = vec![1u32; existing];
        next_energies.extend(reinforcement_energies.iter());
        territories[idx].body_energies = Some(next_energies);
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

    let total_energy: u32 = reinforcement_energies.iter().sum();
    push_log(
        log,
        format!("ターン{}: {}にエナジー{}（合計{}）を増援した。", state.turn, name, count, total_energy),
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
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_attack_action(
    state: &GameState,
    log: &mut Vec<String>,
    from_territory_id: &str,
    to_territory_id: &str,
    count: u32,
    energy_per_body: &Option<Vec<u32>>,
    our_body_names: &Option<Vec<String>>,
    attack_unit_name: &Option<String>,
    speed_per_body: &Option<Vec<u32>>,
    skills_per_body: &Option<Vec<SkillData>>,
    stats_per_body: &Option<Vec<CardStats>>,
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
    let home_id = home_territory_id();
    let home_idx = match get_territory_index(&territories, &home_id) {
        Some(index) => index,
        None => return state.clone(),
    };
    if from_idx == to_idx {
        return state.clone();
    }
    if is_home_territory(to_territory_id) {
        return state.clone();
    }
    if territories[from_idx].owner_id.as_deref() != Some("player") {
        return state.clone();
    }
    if territories[home_idx].troops < count || count == 0 {
        return state.clone();
    }
    if !is_attackable_target(&territories, to_territory_id) {
        return state.clone();
    }

    let from_name = territory_name(&territories, from_territory_id).to_string();
    let to_name = territory_name(&territories, to_territory_id).to_string();
    let to_troops = territories[to_idx].troops;
    let facility_bonuses = crate::facilities::calculate_facility_bonuses(&state.facilities);

    let our_energies: Vec<u32> = energy_per_body
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

    let mut our_chars: Vec<CombatCharacter> = our_names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let stats = our_stats.get(index).cloned().unwrap_or_default();
            let base_energy = if stats.energy > 0 { stats.energy } else { *our_energies.get(index).unwrap_or(&1) };
            let base_speed = if stats.speed > 0 { stats.speed } else { *our_speeds.get(index).unwrap_or(&5) };
            let attack = if stats.attack > 0 { stats.attack } else { 5 };
            let magic = if stats.magic > 0 { stats.magic } else { 5 };
            let defense = if stats.defense > 0 { stats.defense } else { 3 };
            let magic_defense = if stats.magic_defense > 0 { stats.magic_defense } else { 3 };
            let skills = our_skills.get(index).cloned().unwrap_or_default();

            let boosted_energy = crate::facilities::apply_energy_bonus(base_energy, &facility_bonuses);
            let boosted_speed = base_speed + facility_bonuses.speed_bonus;

            CombatCharacter::with_stats(
                index,
                name.clone(),
                boosted_energy,
                boosted_speed,
                attack,
                magic,
                defense,
                magic_defense,
                skills,
            )
        })
        .collect();

    let enemy_energies: Vec<u32> = territories[to_idx]
        .body_energies
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
            let energy = *enemy_energies.get(index).unwrap_or(&1);
            let card_name = name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C');
            if let Some(card) = crate::cards::get_card_by_name(card_name) {
                CombatCharacter::with_stats(
                    index + 100,
                    name.clone(),
                    energy,
                    card.stats.speed,
                    card.stats.attack,
                    card.stats.magic,
                    card.stats.defense,
                    card.stats.magic_defense,
                    SkillData::default(),
                )
            } else {
                CombatCharacter::new(index + 100, name.clone(), energy, 5, SkillData::default())
            }
        })
        .collect();

    if dev_auto_win {
        let max_enemy = enemy_chars.iter().map(|character| character.base_energy).max().unwrap_or(1);
        for character in our_chars.iter_mut() {
            character.current_energy = (max_enemy + 1) as f32;
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

    push_log(log, "--- スキル発動フェーズ ---".to_string());
    apply_battle_start_skills(&mut our_chars, log);

    push_log(log, "--- 戦闘フェーズ ---".to_string());

    let mut our_idx = 0usize;
    let mut enemy_idx = 0usize;

    while our_idx < our_chars.len() && enemy_idx < enemy_chars.len() {
        if !our_chars[our_idx].is_alive {
            our_idx += 1;
            continue;
        }
        if !enemy_chars[enemy_idx].is_alive {
            enemy_idx += 1;
            continue;
        }

        if our_chars[our_idx].is_disabled() {
            push_log(log, format!("{}は行動不能！", our_chars[our_idx].name));
            our_chars[our_idx].process_turn_effects(log);
            our_idx += 1;
            continue;
        }

        let attack_mods = if our_chars[our_idx].is_silenced() {
            push_log(log, format!("{}は沈黙中でスキル使用不可！", our_chars[our_idx].name));
            crate::skills::AttackModifiers::new()
        } else {
            apply_attack_skills(&mut our_chars[our_idx], log)
        };

        let evasion_rate = enemy_chars[enemy_idx].get_evasion_rate();
        if evasion_rate > 0.0 && rand::random::<f32>() < evasion_rate {
            push_log(log, format!("{}の攻撃を{}が回避！", our_chars[our_idx].name, enemy_chars[enemy_idx].name));
            our_idx += 1;
            continue;
        }

        if enemy_chars[enemy_idx].consume_invincible() {
            push_log(log, format!("{}は無敵で攻撃を無効化！", enemy_chars[enemy_idx].name));
            our_idx += 1;
            continue;
        }

        let base_damage = our_chars[our_idx].current_energy * our_chars[our_idx].damage_multiplier;
        let mut total_damage = (base_damage * attack_mods.damage_multiplier + attack_mods.damage_add).max(0.0);
        total_damage += attack_mods.true_damage;

        if attack_mods.percent_damage > 0.0 {
            let percent_dmg = enemy_chars[enemy_idx].current_energy * attack_mods.percent_damage;
            total_damage += percent_dmg;
            push_log(log, format!("割合ダメージ+{:.0}", percent_dmg));
        }

        let vulnerability = enemy_chars[enemy_idx].get_vulnerability();
        let mark_damage = enemy_chars[enemy_idx].get_mark_damage();
        total_damage = total_damage * (1.0 + vulnerability) + mark_damage;

        let enemy_defense = if attack_mods.ignore_defense {
            0.0
        } else {
            enemy_chars[enemy_idx].current_energy * (1.0 - enemy_chars[enemy_idx].damage_reduction)
        };

        let our_name = our_chars[our_idx].name.clone();
        let enemy_name = enemy_chars[enemy_idx].name.clone();

        push_log(
            log,
            format!(
                "{}が{}に攻撃！（攻撃力{:.0} vs 防御力{:.0}）",
                our_name, enemy_name, total_damage, enemy_defense
            ),
        );

        let hp_ratio = enemy_chars[enemy_idx].current_energy / enemy_chars[enemy_idx].base_energy as f32;
        if attack_mods.execute_threshold > 0.0 && hp_ratio <= attack_mods.execute_threshold {
            enemy_chars[enemy_idx].is_alive = false;
            enemy_chars[enemy_idx].current_energy = 0.0;
            push_log(log, format!("処刑発動！{}を即死させた！", enemy_name));
            enemy_idx += 1;
            our_chars[our_idx].process_turn_effects(log);
            continue;
        }

        if attack_mods.aoe_damage > 0.0 {
            push_log(log, format!("全体攻撃で敵全員に{:.0}ダメージ！", attack_mods.aoe_damage));
            for enemy in enemy_chars.iter_mut() {
                if enemy.is_alive {
                    enemy.current_energy -= attack_mods.aoe_damage;
                    if enemy.current_energy <= 0.0 {
                        enemy.is_alive = false;
                        push_log(log, format!("{}が全体攻撃で撃破されました。", enemy.name));
                    }
                }
            }
        }

        for status_effect in &attack_mods.status_effects {
            apply_effect_to_character(status_effect, &mut enemy_chars[enemy_idx], log);
        }

        for self_effect in &attack_mods.self_effects {
            apply_effect_to_character(self_effect, &mut our_chars[our_idx], log);
        }

        for ally_effect in &attack_mods.ally_effects {
            for ally in our_chars.iter_mut() {
                if ally.is_alive {
                    apply_effect_to_character(ally_effect, ally, log);
                }
            }
        }

        for heal_effect in &attack_mods.heal_effects {
            if let Some(lowest) = our_chars.iter_mut().filter(|character| character.is_alive).min_by(|a, b| {
                a.current_energy
                    .partial_cmp(&b.current_energy)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                apply_effect_to_character(heal_effect, lowest, log);
            }
        }

        if total_damage > enemy_defense {
            let damage_after_shield = enemy_chars[enemy_idx].absorb_damage_with_shield(total_damage - enemy_defense);

            if damage_after_shield > 0.0 || total_damage > enemy_defense {
                enemy_chars[enemy_idx].is_alive = false;
                push_log(log, format!("{}が{}を撃破しました。", our_name, enemy_name));

                if attack_mods.energy_steal > 0.0 {
                    our_chars[our_idx].current_energy += attack_mods.energy_steal;
                    push_log(log, format!("{}が{:.0}エナジーを奪取！", our_name, attack_mods.energy_steal));
                }

                if attack_mods.absorb_rate > 0.0 {
                    let absorb = enemy_defense * attack_mods.absorb_rate;
                    our_chars[our_idx].current_energy += absorb;
                    push_log(log, format!("{}が{:.0}エナジーを吸収！", our_name, absorb));
                }

                if attack_mods.extra_attacks > 0 {
                    our_chars[our_idx].extra_attacks += attack_mods.extra_attacks;
                    push_log(log, format!("{}が追加攻撃権を得た！", our_name));
                }

                enemy_idx += 1;
            } else {
                push_log(log, format!("{}のシールドがダメージを吸収！", enemy_name));
            }
        } else if total_damage < enemy_defense {
            let reflect_rate = enemy_chars[enemy_idx].get_reflect_rate();
            if reflect_rate > 0.0 {
                let reflect_damage = total_damage * reflect_rate;
                our_chars[our_idx].current_energy -= reflect_damage;
                push_log(log, format!("{}の反射で{:.0}ダメージ！", enemy_name, reflect_damage));
            }

            our_chars[our_idx].is_alive = false;
            if !check_death_skills(&mut our_chars[our_idx], log) {
                push_log(log, format!("{}が{}に撃破されました。", our_name, enemy_name));
            }

            let counter_rate = enemy_chars[enemy_idx].get_counter_rate();
            if counter_rate > 0.0 && rand::random::<f32>() < counter_rate {
                push_log(log, format!("{}の反撃！", enemy_name));
            }

            our_idx += 1;
        } else {
            our_chars[our_idx].is_alive = false;
            enemy_chars[enemy_idx].is_alive = false;
            let our_revived = check_death_skills(&mut our_chars[our_idx], log);
            if !our_revived {
                push_log(log, format!("相打ち。{}と{}が撃破されました。", our_name, enemy_name));
            } else {
                push_log(log, format!("{}が撃破されました。", enemy_name));
            }
            our_idx += 1;
            enemy_idx += 1;
        }

        our_chars[our_idx.saturating_sub(1)].process_turn_effects(log);

        if our_idx > 0 && our_chars[our_idx - 1].extra_attacks > 0 && our_chars[our_idx - 1].is_alive {
            our_chars[our_idx - 1].extra_attacks -= 1;
            our_idx -= 1;
        }
    }

    let surviving_allies: Vec<&CombatCharacter> = our_chars.iter().filter(|character| character.is_alive).collect();
    let surviving_enemies: Vec<&CombatCharacter> = enemy_chars.iter().filter(|character| character.is_alive).collect();

    let conquered = surviving_enemies.is_empty();
    let mut new_inventory = state.inventory.clone();
    let mut new_owned_cards = state.owned_cards.clone();

    if conquered {
        territories[to_idx].owner_id = Some("player".to_string());
        let occupying: Vec<u32> = if surviving_allies.is_empty() {
            vec![1u32]
        } else {
            surviving_allies.iter().map(|character| character.effective_energy()).collect()
        };
        territories[to_idx].troops = occupying.len() as u32;
        territories[to_idx].body_energies = Some(occupying);
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
    } else {
        let remaining_energies: Vec<u32> = surviving_enemies.iter().map(|character| character.effective_energy()).collect();
        let remaining_names: Vec<String> = surviving_enemies.iter().map(|character| character.name.clone()).collect();
        territories[to_idx].troops = remaining_energies.len() as u32;
        territories[to_idx].body_energies = Some(remaining_energies);
        territories[to_idx].body_names = if remaining_names.is_empty() { None } else { Some(remaining_names) };
        push_log(log, format!("攻撃失敗。{}の防衛に成功。", to_name));
    }

    let mut new_players = state.players.clone();
    if let Some(player) = new_players.get_mut(DEFAULT_PLAYER_ID) {
        player.inventory = new_inventory.clone();
        player.owned_cards = new_owned_cards.clone();
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
    )
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
