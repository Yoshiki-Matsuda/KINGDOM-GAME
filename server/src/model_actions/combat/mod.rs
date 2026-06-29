mod bonus;
mod damage;
mod resolve;

pub(super) use bonus::*;
pub(super) use damage::*;
pub(super) use resolve::*;
use super::*;
use crate::model::{
    push_system_event, push_battle_start_event, push_battle_end_event,
    push_attack_event, push_defeat_event, push_absorb_event, push_enemy_roster_event,
    push_phase_event, push_skill_effect_event, push_conquest_event, push_conquest_reward_event,
    push_card_drop_event, push_ruin_clear_event,
};

fn apply_battle_loot_drops(
    actor_player_id: &str,
    working_players: &mut std::collections::HashMap<String, crate::model::PlayerData>,
    inventory: &mut Vec<crate::items::InventoryItem>,
    drops: Vec<crate::items::InventoryItem>,
    log: &mut Vec<GameEvent>,
) {
    if drops.is_empty() {
        return;
    }
    push_phase_event(log, "戦利品");
    let Some(gold) = working_players
        .get_mut(actor_player_id)
        .map(|p| &mut p.resources.gold)
    else {
        return;
    };
    crate::items::apply_item_rewards(inventory, gold, actor_player_id, drops, log);
}

pub(super) fn apply_deploy_action(
    state: &GameState,
    log: &mut Vec<GameEvent>,
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
    push_system_event(log, &format!("{}に魔獣数{}（合計{}）を増援した。", name, count, total_monster_count));

    build_game_state(state, territories, log.clone(), state.players.clone())
}


#[allow(clippy::too_many_arguments)]
pub(super) fn apply_attack_action(
    state: &GameState,
    log: &mut Vec<GameEvent>,
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
    skip_home_body_check: bool,
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
    let attack_own_territory = territories[to_idx].owner_id.as_deref() == Some(actor_player_id);
    let use_neutral_combat = attack_target_neutral || attack_own_territory;
    let home_id = home_territory_id();
    if get_territory_index(&territories, &home_id).is_none() {
        return state.clone();
    }
    if from_idx == to_idx {
        return state.clone();
    }
    if is_home_territory(to_territory_id) {
        push_system_event(log, "本拠地は攻撃できません。");
        return state.clone();
    }
    if territories[from_idx].owner_id.as_deref() != Some(actor_player_id) {
        push_system_event(log, &format!("出撃元({})を所有していません。", from_territory_id));
        return state.clone();
    }
    // 本拠編成（owned_card_indices）の遠征: from は隣接ルート用のみ。編成体数は本拠で判定する
    let expedition_from_home = owned_card_indices.is_some();
    if expedition_from_home && !skip_home_body_check {
        let Some(player) = state.players.get(actor_player_id) else {
            return state.clone();
        };
        let away = super::march::march_bodies_away_count(player, default_now_ms());
        let cap = player.owned_cards.len() as u32;
        let available = cap.saturating_sub(away);
        if count == 0 || count > available {
            let home_id = player.home_territory_id.as_str();
            push_system_event(log, &format!(
                    "本拠({})の編成体数が足りません（必要{}体, 利用可能{}体）。",
                    home_id, count, available
                ));
            return state.clone();
        }
    } else if !expedition_from_home && (territories[from_idx].troops < count || count == 0) {
        push_system_event(log, &format!(
                "出撃元({})の駐留体数が足りません（必要{}体, 現在{}体）。",
                from_territory_id, count, territories[from_idx].troops
            ));
        return state.clone();
    }
    let base_owners = attack_base_owner_ids(state, actor_player_id);
    if !is_attackable_target(&territories, to_territory_id, &base_owners) {
        push_system_event(log, &format!("{}は攻撃対象外です（隣接領地なし）。", to_territory_id));
        return state.clone();
    }
    if !territories_are_adjacent(from_territory_id, to_territory_id) {
        push_system_event(log, &format!("{}と{}は隣接していません。", from_territory_id, to_territory_id));
        return state.clone();
    }

    let march_monster_counts: Vec<u32> = monsters_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![1u32; count as usize]);
    let our_stats: Vec<CardStats> = if let (Some(oci), Some(player)) = (
        owned_card_indices
            .as_ref()
            .filter(|indices| indices.len() == count as usize),
        state.players.get(actor_player_id),
    ) {
        let mc: Vec<u32> = oci
            .iter()
            .enumerate()
            .map(|(body_i, &slot)| {
                march_monster_counts
                    .get(body_i)
                    .copied()
                    .or_else(|| player.card_monster_counts.get(slot).copied())
                    .unwrap_or(1)
            })
            .collect();
        crate::model::resolve_authoritative_body_stats(player, oci, Some(&mc))
    } else {
        stats_per_body
            .clone()
            .filter(|values| values.len() == count as usize)
            .unwrap_or_else(|| vec![CardStats::default(); count as usize])
    };
    let our_body_monster_counts: Vec<u32> = our_stats
        .iter()
        .map(|s| s.monster_count.max(1))
        .collect();
    let our_speeds: Vec<u32> = speed_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| our_stats.iter().map(|s| s.speed.max(1)).collect());
    let our_names: Vec<String> = our_body_names
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| (1..=count as usize).map(|i| format!("味方ユニット{}", i)).collect());
    let our_skills: Vec<SkillData> = skills_per_body
        .clone()
        .filter(|values| values.len() == count as usize)
        .unwrap_or_else(|| vec![SkillData::default(); count as usize]);

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
        push_system_event(log, &format!(
                "ユニットコスト上限({:.1})を超えています（編成コスト合計{:.1}）。",
                cost_cap, total_cost
            ));
        return state.clone();
    }

    let mut working_players = state.players.clone();

    if let Some(ref oci) = owned_card_indices {
        if oci.len() != count as usize {
            return state.clone();
        }
        let Some(player) = working_players.get(actor_player_id) else {
            return state.clone();
        };
        for &i in oci {
            if i >= player.owned_cards.len() {
                push_system_event(log, "無効な魔獣スロットです。");
                return state.clone();
            }
        }
        let mut seen_slots = HashSet::new();
        for &i in oci {
            if !seen_slots.insert(i) {
                push_system_event(log, "同一体スロットを複数回指定できません。");
                return state.clone();
            }
        }
        let mut seen_card_ids = HashSet::new();
        for &i in oci {
            let card_id = player.owned_cards[i];
            if !seen_card_ids.insert(card_id) {
                push_system_event(log, "同一種の魔獣を複数配置できません。");
                return state.clone();
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

    let (_defender_troops, enemy_monster_counts, enemy_names) = if attack_own_territory {
        generate_neutral_enemies_for_territory(territories[to_idx].level, &territories[to_idx].id)
    } else {
        resolve_territory_defenders(&territories[to_idx], &state.players)
    };

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
    let defender_owner = territories[to_idx].owner_id.as_deref();
    let defender_label = defender_owner
        .and_then(|owner_id| {
            state
                .ai_factions
                .iter()
                .find(|f| format!("ai_{}", f.faction_id) == owner_id)
                .map(|f| f.name.clone())
        })
        .or_else(|| defender_owner.map(|id| id.to_string()))
        .unwrap_or_else(|| to_name.clone());
    let coords_str = parse_territory_coords(to_territory_id)
        .map(|(col, row)| format!("<{},{}>", col, row))
        .unwrap_or_default();
    push_battle_start_event(log, actor_player_id, &to_name, &defender_label, &coords_str, attacker_label);

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
            if !c.is_alive {
                continue;
            }
            c.status_effects.clear();
            c.damage_multiplier = 1.0;
            c.damage_reduction = 0.0;
            c.extra_attacks = 0;
            if c.startup_monster_factor > 1.0 + f32::EPSILON {
                c.current_monster_count /= c.startup_monster_factor;
                c.startup_monster_factor = 1.0;
            }
            c.current_speed = c.pre_startup_speed;
        }
        push_phase_event(log, &format!("第{}戦", wave));
    }

    if dev_auto_win {
        apply_dev_auto_win_boost(&mut our_chars, &enemy_chars);
    }

    push_enemy_roster_log(log, &enemy_chars);

    push_phase_event(log, "スタートアップフェーズ");
    if wave == 1 {
        apply_race_bonus(&mut our_chars, log);
        apply_race_lab_bonus(&mut our_chars, player_facilities, log);
        apply_shrine_bonus(&mut our_chars, &facility_bonuses, log);
        apply_race_bonus(&mut enemy_chars, log);
    }
    for c in our_chars.iter_mut().filter(|c| c.is_alive) {
        c.pre_startup_speed = c.current_speed;
    }
    for c in enemy_chars.iter_mut().filter(|c| c.is_alive) {
        c.pre_startup_speed = c.current_speed;
    }
    apply_battle_start_skills(&mut our_chars, &mut enemy_chars, log);

    push_phase_event(log, "戦闘フェーズ");

    let max_combat_turns: u32 = 8;
    let mut last_combat_turn: u32 = 0;
    'battle: for combat_turn in 1..=max_combat_turns {
        last_combat_turn = combat_turn;
        push_phase_event(log, &format!("Turn {}", combat_turn));

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
        push_battle_end_event(log, actor_player_id, "timeout", &to_name);
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
    let neutral_won_by_commander = use_neutral_combat
        && !surviving_allies.is_empty()
        && enemy_leader_defeated;
    let wave_won_for_occ = waves_cleared || neutral_won_by_commander;

    let mut conquered = false;
    let mut partial_occupation = false;
    if wave_won_for_occ {
        let occ: u32 = surviving_allies.iter().map(|c| c.occupation_power).sum();
        let max_d = territories[to_idx].max_durability;
        if max_d == 0 || attack_own_territory {
            conquered = true;
        } else {
            territories[to_idx].durability = territories[to_idx].durability.saturating_sub(occ);
            if territories[to_idx].durability == 0 {
                conquered = true;
            } else {
                partial_occupation = true;
                push_battle_end_event(log, actor_player_id, "partial", &to_name);
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
    if conquered && attack_own_territory {
        push_battle_end_event(log, actor_player_id, "practice_victory", &to_name);
        let enemy_type_refs: Vec<&str> = enemy_names.iter().map(|name| name.as_str()).collect();
        let drops = crate::items::calculate_drops(&enemy_type_refs, facility_bonuses.drop_rate);
        apply_battle_loot_drops(actor_player_id, &mut working_players, &mut new_inventory, drops, log);
        let dropped_cards = calculate_card_drops(&enemy_names, facility_bonuses.drop_rate as f32);
        if !dropped_cards.is_empty() {
            push_phase_event(log, "魔獣入手");
            for card_id in &dropped_cards {
                if let Some(card) = crate::cards::get_card(*card_id) {
                    push_card_drop_event(log, actor_player_id, &card.name);
                    new_owned_cards.push(*card_id);
                }
            }
        }
    } else if conquered {
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
        push_battle_end_event(log, actor_player_id, "victory", &to_name);
        push_conquest_event(log, actor_player_id, &to_name);

        let level = territories[to_idx].level;
        let (food, wood, stone, iron) = crate::model::conquest_resource_bonus(level);
        if let Some(player) = working_players.get_mut(actor_player_id) {
            player.resources.food = player.resources.food.saturating_add(food);
            player.resources.wood = player.resources.wood.saturating_add(wood);
            player.resources.stone = player.resources.stone.saturating_add(stone);
            player.resources.iron = player.resources.iron.saturating_add(iron);
            push_conquest_reward_event(log, actor_player_id, food, wood, stone, iron);
        }

        let is_ruin = territories[to_idx].ruin.is_some();
        let enemy_type_refs: Vec<&str> = enemy_names.iter().map(|name| name.as_str()).collect();
        let drops = crate::items::calculate_drops(&enemy_type_refs, facility_bonuses.drop_rate);
        apply_battle_loot_drops(actor_player_id, &mut working_players, &mut new_inventory, drops, log);

        let dropped_cards = calculate_card_drops(&enemy_names, facility_bonuses.drop_rate as f32);
        if !dropped_cards.is_empty() {
            push_phase_event(log, "魔獣入手");
            for card_id in &dropped_cards {
                if let Some(card) = crate::cards::get_card(*card_id) {
                    push_card_drop_event(log, actor_player_id, &card.name);
                    new_owned_cards.push(*card_id);
                }
            }
        }

        if is_ruin {
            territories[to_idx].ruin = None;
            push_ruin_clear_event(log, actor_player_id);
        }
    } else if !partial_occupation {
        if attack_own_territory {
            push_battle_end_event(log, actor_player_id, "practice_defeat", &to_name);
        } else {
            let remaining_monster_counts: Vec<u32> = surviving_enemies.iter().map(|character| character.effective_monster_count()).collect();
            let remaining_names: Vec<String> = surviving_enemies.iter().map(|character| character.name.clone()).collect();
            territories[to_idx].troops = remaining_monster_counts.len() as u32;
            territories[to_idx].body_monster_counts = Some(remaining_monster_counts);
            territories[to_idx].body_names = if remaining_names.is_empty() { None } else { Some(remaining_names) };
            push_battle_end_event(log, actor_player_id, "defeat", &to_name);
        }
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
        if conquered && !attack_own_territory {
            if let Some(ref idxs) = owned_card_indices {
                award_conquest_xp(player, idxs, &facility_bonuses, log);
            }
        }
    }

    // KC仕様: 本拠地陥落 → 対象プレイヤーの所属同盟が、攻撃側の同盟の配下同盟となる
    let mut alliances = state.alliances.clone();
    if conquered && was_base && !attack_own_territory {
        if let Some(prev) = prev_owner_id.as_deref() {
            subjugate_alliance_of(prev, actor_player_id, &mut alliances, log);
        }
    }

    let mut out = build_game_state(state, territories, log.clone(), new_players);
    out.alliances = alliances;
    out
}
