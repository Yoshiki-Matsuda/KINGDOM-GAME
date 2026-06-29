use super::*;

/// 攻撃遠征の編成（dev_bot と同様に stats / 魔獣数を Action に載せる）
#[derive(Debug, Clone)]
pub struct AttackFormation {
    pub card_indices: Vec<usize>,
    pub monster_counts: Vec<u32>,
    pub body_names: Vec<String>,
    pub speeds: Vec<u32>,
    pub skills: Vec<SkillData>,
    pub stats: Vec<CardStats>,
}

/// KC 編成スロット [前衛, 中衛, リーダー] に本拠スロット列を割り当てる
pub(crate) fn slots_to_formed_indices(slots: &[usize]) -> [i32; 3] {
    let mut out = [-1i32; 3];
    match slots.len() {
        1 => out[2] = slots[0] as i32,
        2 => {
            out[0] = slots[0] as i32;
            out[2] = slots[1] as i32;
        }
        3 => {
            out[0] = slots[0] as i32;
            out[1] = slots[1] as i32;
            out[2] = slots[2] as i32;
        }
        _ => {}
    }
    out
}

pub(crate) fn ai_formed_units_valid(player: &PlayerData, max_units: usize, now: u64) -> bool {
    if player.formed_units.is_empty() || player.formed_units.len() > max_units {
        return false;
    }
    let locked = march_locked_card_slots(player, now);
    let mut used = HashSet::new();
    for unit in &player.formed_units {
        if !is_kc_unit_ready_to_deploy(&unit.indices) {
            return false;
        }
        for slot in formation_owned_slots_in_slot_order(&unit.indices) {
            if slot >= player.owned_cards.len() || locked.contains(&slot) || !used.insert(slot) {
                return false;
            }
        }
    }
    true
}

pub(crate) fn best_unit_combo(player: &PlayerData, candidates: &[usize], cost_cap: f32) -> Option<Vec<usize>> {
    let max_bodies = candidates.len().min(3);
    let mut best: Option<(i64, Vec<usize>)> = None;
    for size in 1..=max_bodies {
        pick_combinations(candidates, size, 0, &mut Vec::new(), &mut |combo: &[usize]| {
            let mut species = HashSet::new();
            for &idx in combo {
                if !species.insert(player.owned_cards[idx]) {
                    return;
                }
            }
            if formation_total_cost(player, combo) > cost_cap + 0.0001 {
                return;
            }
            let score: i64 = combo
                .iter()
                .map(|&i| player.card_monster_counts.get(i).copied().unwrap_or(1) as i64)
                .sum();
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, combo.to_vec()));
            }
        });
    }
    best.map(|(_, combo)| combo)
}

pub(crate) fn build_ai_formed_units(
    player: &PlayerData,
    max_units: usize,
    cost_cap: f32,
    now: u64,
) -> Vec<StoredFormedUnit> {
    let locked = march_locked_card_slots(player, now);
    let mut free: Vec<usize> = (0..player.owned_cards.len())
        .filter(|i| !locked.contains(i))
        .collect();
    let mut used_slots = HashSet::new();
    let mut units = Vec::new();

    for unit_idx in 0..max_units {
        let candidates: Vec<usize> = free
            .iter()
            .copied()
            .filter(|i| !used_slots.contains(i))
            .collect();
        if candidates.is_empty() {
            break;
        }
        let Some(combo) = best_unit_combo(player, &candidates, cost_cap) else {
            break;
        };
        for &i in &combo {
            used_slots.insert(i);
            if let Some(pos) = free.iter().position(|&x| x == i) {
                free.remove(pos);
            }
        }
        units.push(StoredFormedUnit {
            id: format!("ai-unit-{unit_idx}"),
            name: format!("ユニット{}", unit_idx + 1),
            indices: slots_to_formed_indices(&combo),
        });
    }
    units
}

/// AI のユニット編成を施設上限まで維持（プレイヤーの `set_formed_units` と同じデータ）
pub(crate) fn ensure_ai_formed_units(player: &mut PlayerData, now: u64) {
    let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
    let max_units = (1 + bonuses.unit_capacity).max(1) as usize;
    let cost_cap = player.unit_cost_cap + bonuses.unit_cost_cap_bonus;
    if ai_formed_units_valid(player, max_units, now) {
        return;
    }
    player.formed_units = build_ai_formed_units(player, max_units, cost_cap, now);
}

pub(crate) fn initialize_ai_formed_units(player: &mut PlayerData) {
    ensure_ai_formed_units(player, default_now_ms());
}

pub(crate) fn formed_unit_available_for_dispatch(player: &PlayerData, unit: &StoredFormedUnit, now: u64) -> bool {
    if !is_kc_unit_ready_to_deploy(&unit.indices) {
        return false;
    }
    if march_busy_formed_unit_ids(player, now).contains(&unit.id) {
        return false;
    }
    let locked = march_locked_card_slots(player, now);
    formation_owned_slots_in_slot_order(&unit.indices)
        .iter()
        .all(|&slot| slot < player.owned_cards.len() && !locked.contains(&slot))
}

pub(crate) fn pick_ai_attack_from_formed_units(
    state: &GameState,
    ai_id: &str,
    target: &Territory,
    now: u64,
) -> Option<(String, AttackFormation)> {
    let player = state.players.get(ai_id)?;
    let (_, def_counts, def_names) = resolve_territory_defenders(target, &state.players);
    let mut ranked: Vec<(i64, String, AttackFormation)> = player
        .formed_units
        .iter()
        .filter(|u| formed_unit_available_for_dispatch(player, u, now))
        .filter_map(|u| {
            let slots = formation_owned_slots_in_slot_order(&u.indices);
            let formation = build_formation_from_indices(player, &slots)?;
            let score = score_formation(&slots, player, &def_names, &def_counts);
            Some((score, u.id.clone(), formation))
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, unit_id, formation) in ranked {
        let player = state.players.get(ai_id)?;
        if formation_viable_for_target(state, &formation, player, target) {
            return Some((unit_id, formation));
        }
    }
    None
}

pub(crate) fn unit_cost_cap_for(state: &GameState, player_id: &str) -> f32 {
    let facilities = state
        .players
        .get(player_id)
        .map(|p| p.facilities.as_slice())
        .unwrap_or(&[]);
    let bonuses = crate::facilities::calculate_facility_bonuses(facilities);
    state
        .players
        .get(player_id)
        .map(|p| p.unit_cost_cap)
        .unwrap_or(4.0)
        + bonuses.unit_cost_cap_bonus
}

pub(crate) fn card_effective_cost(
    player: &crate::model::PlayerData,
    card_index: usize,
) -> f32 {
    let card_id = player.owned_cards.get(card_index).copied().unwrap_or(0);
    let enhanced = player.card_enhanced.get(card_index).copied().unwrap_or(false);
    let base = get_card(card_id).map(|c| c.stats.cost).unwrap_or(1.5);
    if enhanced {
        base * 0.75
    } else {
        base
    }
}

pub(crate) fn formation_total_cost(
    player: &crate::model::PlayerData,
    indices: &[usize],
) -> f32 {
    indices
        .iter()
        .map(|&i| card_effective_cost(player, i))
        .sum()
}

/// 攻撃目標に対する最適編成（ユニットコスト上限内・1〜3体・同一種不可）
pub fn build_attack_formation(
    state: &GameState,
    ai_id: &str,
    target: &Territory,
) -> Option<AttackFormation> {
    let player = state.players.get(ai_id)?;
    let cost_cap = unit_cost_cap_for(state, ai_id);
    let now = crate::model::default_now_ms();
    let (_, def_counts, def_names) = resolve_territory_defenders(target, &state.players);

    let mut candidates: Vec<usize> = Vec::new();
    for i in 0..player.owned_cards.len() {
        if i < player.card_rest_until.len() && player.card_rest_until[i] > now {
            continue;
        }
        let stamina = player.card_stamina.get(i).copied().unwrap_or(0);
        if stamina < STAMINA_ATTACK {
            continue;
        }
        let count = player.card_monster_counts.get(i).copied().unwrap_or(0);
        if count == 0 {
            continue;
        }
        candidates.push(i);
    }
    if candidates.is_empty() {
        return None;
    }

    let max_bodies = candidates.len().min(3);
    let mut best: Option<(i64, Vec<usize>)> = None;

    for size in 1..=max_bodies {
        pick_combinations(&candidates, size, 0, &mut Vec::new(), &mut |combo: &[usize]| {
            let mut species = HashSet::new();
            for &idx in combo {
                if !species.insert(player.owned_cards[idx]) {
                    return;
                }
            }
            if formation_total_cost(player, combo) > cost_cap + 0.0001 {
                return;
            }
            let score = score_formation(combo, player, &def_names, &def_counts);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, combo.to_vec()));
            }
        });
    }

    let indices = best.map(|(_, idx)| idx)?;
    build_formation_from_indices(player, &indices)
}

pub(crate) fn pick_combinations<F>(candidates: &[usize], size: usize, start: usize, current: &mut Vec<usize>, f: &mut F)
where
    F: FnMut(&[usize]),
{
    if current.len() == size {
        f(current);
        return;
    }
    let need = size - current.len();
    for i in start..=candidates.len().saturating_sub(need) {
        current.push(candidates[i]);
        pick_combinations(candidates, size, i + 1, current, f);
        current.pop();
    }
}

pub(crate) fn build_formation_from_indices(
    player: &crate::model::PlayerData,
    indices: &[usize],
) -> Option<AttackFormation> {
    let mut card_indices = Vec::new();
    let mut monster_counts = Vec::new();
    let mut body_names = Vec::new();
    let mut speeds = Vec::new();
    let mut skills = Vec::new();
    let mut stats = Vec::new();

    for &idx in indices {
        let card_id = player.owned_cards.get(idx).copied()?;
        let card = get_card(card_id)?;
        let mc = player
            .card_monster_counts
            .get(idx)
            .copied()
            .unwrap_or(card.stats.monster_count)
            .max(1);
        card_indices.push(idx);
        monster_counts.push(mc);
        body_names.push(card.name.to_string());
        speeds.push(card.stats.speed);
        skills.push(get_card_skills(card_id));
        stats.push(card.stats.clone());
    }

    Some(AttackFormation {
        card_indices,
        monster_counts,
        body_names,
        speeds,
        skills,
        stats,
    })
}

pub(crate) fn score_formation(
    indices: &[usize],
    player: &crate::model::PlayerData,
    def_names: &[String],
    def_counts: &[u32],
) -> i64 {
    let mut score = 0i64;
    let mut species = HashSet::new();
    for &idx in indices {
        let card_id = player.owned_cards.get(idx).copied().unwrap_or(0);
        if !species.insert(card_id) {
            score -= 50;
        }
        let card = get_card(card_id);
        let atk = card.map(|c| c.stats.attack as i64).unwrap_or(10);
        let hp = card.map(|c| c.stats.defense as i64).unwrap_or(100);
        score += atk + hp / 10;
        let mc = player.card_monster_counts.get(idx).copied().unwrap_or(1) as i64;
        score += mc.min(500);
    }
    for (i, name) in def_names.iter().enumerate() {
        let def_power = def_counts.get(i).copied().unwrap_or(1) as i64 * 5;
        for &idx in indices {
            let card_id = player.owned_cards.get(idx).copied().unwrap_or(0);
            if counter_bonus(card_id, name) {
                score += 30 + def_power;
            }
        }
    }
    score
}

pub(crate) fn counter_bonus(attacker_card_id: u32, defender_name: &str) -> bool {
    let atk_name = get_card(attacker_card_id)
        .map(|c| c.name)
        .unwrap_or("");
    if defender_name.contains("スケルトン") && atk_name.contains("オーディン") {
        return true;
    }
    if defender_name.contains("ゴブリン") && attacker_card_id >= 12 {
        return true;
    }
    false
}

