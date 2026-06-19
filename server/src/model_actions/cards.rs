use super::*;
use crate::model::push_system_event;

/// 魔獣1体あたりの食料コスト（生産）
const FOOD_PER_MONSTER_PRODUCE: u64 = 2;

pub(super) fn apply_produce_monsters(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    card_index: usize,
    amount: u32,
) -> GameState {
    if amount == 0 {
        push_system_event(log, "生産量は1以上にしてください。");
        return state.clone();
    }
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };
    ensure_card_monster_counts(player);
    if card_index >= player.owned_cards.len() {
        push_system_event(log, "無効な魔獣スロットです。");
        return state.clone();
    }
    let now = default_now_ms();
    if super::march::march_locked_card_slots(player, now).contains(&card_index) {
        push_system_event(log, "遠征中の魔獣は生産できません。");
        return state.clone();
    }
    let card_id = player.owned_cards[card_index];
    let cap = crate::model::MAX_MONSTER_COUNT_PER_CARD_SLOT;
    let cur = player
        .card_monster_counts
        .get(card_index)
        .copied()
        .unwrap_or(1)
        .min(cap);
    let room = cap.saturating_sub(cur);
    let add = amount.min(room);
    if add == 0 {
        push_system_event(log, &format!(
                "これ以上魔獣を生産できません（1スロットあたり上限{}体）。",
                cap
            ));
        return state.clone();
    }
    let food_cost = (add as u64).saturating_mul(FOOD_PER_MONSTER_PRODUCE);
    if player.resources.food < food_cost {
        push_system_event(log, "食料が足りません。");
        return state.clone();
    }
    player.resources.food -= food_cost;
    player.card_monster_counts[card_index] = cur.saturating_add(add);
    let card_name = crate::cards::get_card(card_id)
        .map(|c| c.name.to_string())
        .unwrap_or_else(|| format!("魔獣#{}", card_id));
    push_system_event(log, &format!(
            "「{}」に魔獣を{}体生産した（食料{}を消費）。",
            card_name, add, food_cost
        ));

    let mut territories = state.territories.clone();
    sync_home_territory_body_counts_from_player(&mut territories, player);

    build_game_state(state, territories, log.clone(), players)
}

/// KC準拠合成: 素材魔獣を消費してベース魔獣のスキルレベルアップ
pub(super) fn apply_synthesize_card(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    base_idx: usize,
    material_indices: &[usize],
) -> GameState {
    let mut players = state.players.clone();
    let Some(current_player) = players.get(actor_player_id).cloned() else {
        return state.clone();
    };
    let mut owned_cards = current_player.owned_cards.clone();
    if base_idx >= owned_cards.len() { return state.clone(); }
    if material_indices.is_empty() { return state.clone(); }
    for &idx in material_indices {
        if idx >= owned_cards.len() || idx == base_idx { return state.clone(); }
    }

    let base_card_id = owned_cards[base_idx];
    let base_name = crate::cards::get_card(base_card_id)
        .map(|c| c.name.to_string())
        .unwrap_or_else(|| format!("魔獣#{}", base_card_id));

    let material_count = material_indices.len();
    let levels_before = current_player
        .card_skill_levels
        .get(&base_idx)
        .copied()
        .unwrap_or([1, 1, 1])
        .map(|lv| if lv == 0 { 1 } else { lv.clamp(1, 10) });
    let min_skill_level = levels_before.iter().copied().min().unwrap_or(1);
    // Lv7到達までは1素材あたり+2（例: Lv1→7は素材3枚）、Lv8以降は+1
    let level_up = if min_skill_level < 7 {
        ((material_count as u8).saturating_mul(2)).min(9)
    } else {
        (material_count as u8).min(9)
    };

    push_system_event(log, &format!(
        "「{}」に素材{}枚を合成！スキルLv+{}",
        base_name, material_count, level_up
    ));

    let mut sorted_removals: Vec<usize> = material_indices.to_vec();
    sorted_removals.sort();

    let mut card_monster_counts = {
        let Some(player) = players.get_mut(actor_player_id) else {
            return state.clone();
        };
        ensure_card_monster_counts(player);
        let c = player.card_monster_counts.clone();
        if c.len() != owned_cards.len() {
            initial_card_monster_counts_for_owned(&owned_cards)
        } else {
            c
        }
    };
    remove_indices_from_parallel_vec(&mut card_monster_counts, &sorted_removals);

    let mut to_remove: Vec<usize> = material_indices.to_vec();
    to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for idx in to_remove {
        owned_cards.remove(idx);
    }

    let mut card_skill_levels = std::collections::HashMap::new();
    for (&old_idx, &levels) in &current_player.card_skill_levels {
        if material_indices.contains(&old_idx) { continue; }
        let shift = sorted_removals.iter().filter(|&&r| r < old_idx).count();
        card_skill_levels.insert(old_idx - shift, levels);
    }

    let new_base_idx = base_idx - sorted_removals.iter().filter(|&&r| r < base_idx).count();
    let levels = card_skill_levels.entry(new_base_idx).or_insert([0u8; 3]);
    for lv in levels.iter_mut() {
        *lv = (*lv + level_up).min(10);
    }

    let original_owned_len = current_player.owned_cards.len();
    let mut card_levels = current_player.card_levels.clone();
    let mut card_exp = current_player.card_exp.clone();
    let mut card_stamina = current_player.card_stamina.clone();
    let mut card_status_points = current_player.card_status_points.clone();
    let mut card_stat_bonuses = current_player.card_stat_bonuses.clone();
    let mut card_rest_until = current_player.card_rest_until.clone();
    let mut card_awakened = current_player.card_awakened.clone();
    let mut card_enhanced = current_player.card_enhanced.clone();
    if card_levels.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_levels, &sorted_removals);
    }
    if card_exp.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_exp, &sorted_removals);
    }
    if card_stamina.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_stamina, &sorted_removals);
    }
    if card_status_points.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_status_points, &sorted_removals);
    }
    if card_stat_bonuses.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_stat_bonuses, &sorted_removals);
    }
    if card_rest_until.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_rest_until, &sorted_removals);
    }
    if card_awakened.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_awakened, &sorted_removals);
    }
    if card_enhanced.len() == original_owned_len {
        remove_indices_from_parallel_vec(&mut card_enhanced, &sorted_removals);
    }

    // KC仕様: ベース魔獣の3スキル全てLv5以上 → 合成時に5%で「魔獣覚醒」でLv99超えへ
    if let Some(lvs) = card_skill_levels.get(&new_base_idx) {
        let all_over_5 = lvs.iter().all(|&l| l >= 5);
        if all_over_5 {
            while card_awakened.len() <= new_base_idx {
                card_awakened.push(false);
            }
            if !card_awakened[new_base_idx] {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                if rng.gen::<f32>() < 0.05 {
                    card_awakened[new_base_idx] = true;
                    push_system_event(log, &format!("★ {} が覚醒した！Lv99を超えて成長できる！", base_name));
                }
            }
        }
    }

    // KC仕様: 同一魔獣を素材として重ねる場合の ★化（強化魔獣化）
    // 本実装では、素材に同一 card_id の魔獣を3体以上使った場合に強化魔獣化（低確率）。
    {
        let same_material_count = material_indices
            .iter()
            .filter(|&&i| {
                current_player.owned_cards.get(i).copied() == Some(base_card_id)
            })
            .count();
        if same_material_count >= 3 {
            while card_enhanced.len() <= new_base_idx {
                card_enhanced.push(false);
            }
            if !card_enhanced[new_base_idx] {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let prob = (0.10 + 0.05 * (same_material_count as f32 - 3.0)).min(0.50);
                if rng.gen::<f32>() < prob {
                    card_enhanced[new_base_idx] = true;
                    push_system_event(log, &format!("★ {} は強化魔獣となった！（ステータス+10% / コスト-25%）", base_name));
                }
            }
        }
    }

    if let Some(player) = players.get_mut(actor_player_id) {
        player.owned_cards = owned_cards.clone();
        player.card_skill_levels = card_skill_levels.clone();
        player.card_levels = card_levels.clone();
        player.card_exp = card_exp.clone();
        player.card_stamina = card_stamina.clone();
        player.card_status_points = card_status_points.clone();
        player.card_stat_bonuses = card_stat_bonuses.clone();
        player.card_rest_until = card_rest_until.clone();
        player.card_awakened = card_awakened.clone();
        player.card_enhanced = card_enhanced.clone();
        player.card_monster_counts = card_monster_counts.clone();
    }

    let mut territories = state.territories.clone();
    if let Some(p) = players.get(actor_player_id) {
        sync_home_territory_body_counts_from_player(&mut territories, p);
    }

    build_game_state(state, territories, log.clone(), players)
}

pub(super) fn calculate_card_drops(enemy_names: &[String], drop_rate_bonus: f32) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    let mut dropped = Vec::new();

    for name in enemy_names {
        if let Some(card_id) = crate::cards::get_card_id_by_name(name) {
            let Some(card) = crate::cards::get_card(card_id) else {
                continue;
            };
            let base_chance = crate::cards::get_card_drop_chance(card.rarity);
            let actual_chance = base_chance * (1.0 + drop_rate_bonus);
            if rng.gen::<f32>() < actual_chance && crate::cards::card_has_illustration(card_id) {
                dropped.push(card_id);
            }
        }
    }

    dropped
}
