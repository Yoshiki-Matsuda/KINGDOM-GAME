use super::*;

/// KC準拠: 探索1ミッションで必要なスタミナ
const STAMINA_EXPLORATION: u32 = 50;
/// KC準拠: 探索でスロットあたり加算される同時派遣数の閾値
/// 1〜19 → 1 / 20〜39 → 2 / 40〜59 → 3 / 60〜79 → 4 / 80〜99 → 5 / 100+ → 6
fn exploration_max_slots(exploration_level: u32) -> usize {
    match exploration_level {
        0..=19 => 1,
        20..=39 => 2,
        40..=59 => 3,
        60..=79 => 4,
        80..=99 => 5,
        _ => 6,
    }
}

pub(super) fn apply_start_exploration(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    territory_id: &str,
    card_indices: &[usize],
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };
    if get_territory_index(&state.territories, territory_id).is_none() {
        return state.clone();
    }
    let tidx = get_territory_index(&state.territories, territory_id).unwrap();
    // KC: 本拠地/拠点/塔以外の全ての土地。ただしプロジェクトの仕様として
    // 「占領済みの自領地から魔獣を派遣する」簡略化版を維持（本拠地のみ除外）
    if is_home_territory(territory_id) {
        push_log(log, "本拠地からは探索を派遣しません。".to_string());
        return state.clone();
    }
    if state.territories[tidx].is_base {
        push_log(log, "拠点や塔からは探索を派遣できません。".to_string());
        return state.clone();
    }
    if card_indices.is_empty() {
        push_log(log, "探索に使用する魔獣を選んでください。".to_string());
        return state.clone();
    }
    let max_slots = exploration_max_slots(player.exploration_level);
    if card_indices.len() > max_slots {
        push_log(
            log,
            format!(
                "同時派遣数が探索レベル({}体まで)を超えています。",
                max_slots
            ),
        );
        return state.clone();
    }
    for &i in card_indices {
        if i >= player.owned_cards.len() {
            push_log(log, "無効な魔獣スロットです。".to_string());
            return state.clone();
        }
    }
    if player.explorations.len() + card_indices.len() > max_slots {
        push_log(log, "これ以上探索を出せません。".to_string());
        return state.clone();
    }
    // 事前チェック: 休息中・スタミナ不足・重複
    let now_precheck = default_now_ms();
    let mut used = std::collections::HashSet::new();
    for &i in card_indices {
        if !used.insert(i) {
            push_log(log, "同じ魔獣を重複指定できません。".to_string());
            return state.clone();
        }
        let rest_until = player.card_rest_until.get(i).copied().unwrap_or(0);
        if rest_until > now_precheck {
            push_log(log, "休息中の魔獣を派遣できません。".to_string());
            return state.clone();
        }
        let st = player.card_stamina.get(i).copied().unwrap_or(120);
        if st < STAMINA_EXPLORATION {
            push_log(log, "スタミナが足りない魔獣が含まれています。".to_string());
            return state.clone();
        }
    }
    // スタミナ消費
    while player.card_stamina.len() < player.owned_cards.len() {
        player.card_stamina.push(120);
    }
    for &i in card_indices {
        player.card_stamina[i] = player.card_stamina[i].saturating_sub(STAMINA_EXPLORATION);
    }

    let now = default_now_ms();
    let mission_id = format!("exp-{}", now);
    let territory_name = crate::model::territory_name(&state.territories, territory_id).to_string();
    // KC準拠: 約30分固定。開発用に短縮（3分）。
    player.explorations.push(ExplorationMission {
        mission_id: mission_id.clone(),
        territory_id: territory_id.to_string(),
        started_at: now,
        completes_at: now.saturating_add(3 * 60 * 1000),
        card_indices: card_indices.to_vec(),
    });
    push_log(
        log,
        format!("{} へ探索隊を派遣しました（完了まで約3分）。", territory_name),
    );
    build_game_state(state, state.turn, state.territories.clone(), log.clone(), players)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExplorationOutcome {
    Failure,
    Alive,
    Succeed,
    Excellent,
}

/// 探索結果を判定（KC準拠: スタミナと魔獣Lv vs 土地Lv の差で判定）
fn determine_exploration_outcome(
    avg_stamina: u32,
    avg_card_level: u32,
    territory_level: u8,
) -> ExplorationOutcome {
    let lvl_diff = avg_card_level as i32 - territory_level as i32;
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();

    // スタミナが低すぎるとFAILURE率が上がる
    let base_fail: f32 = if avg_stamina < 40 {
        0.35
    } else if avg_stamina < 80 {
        0.15
    } else {
        0.05
    };
    // 土地レベルが魔獣より高いと失敗率上乗せ
    let penalty: f32 = if lvl_diff < -3 {
        0.30
    } else if lvl_diff < 0 {
        0.10
    } else {
        0.0
    };
    let fail_chance: f32 = (base_fail + penalty).min(0.85_f32);
    if roll < fail_chance {
        return ExplorationOutcome::Failure;
    }

    // EXCELLENT: 魔獣Lvが土地Lvより高く、スタミナもMAXなら確率アップ
    let excellent_chance = if lvl_diff >= 5 && avg_stamina >= 100 {
        0.25
    } else if lvl_diff >= 2 && avg_stamina >= 80 {
        0.10
    } else {
        0.03
    };
    if roll < fail_chance + excellent_chance {
        return ExplorationOutcome::Excellent;
    }
    let succeed_chance = 0.55;
    if roll < fail_chance + excellent_chance + succeed_chance {
        ExplorationOutcome::Succeed
    } else {
        ExplorationOutcome::Alive
    }
}

pub(super) fn apply_collect_exploration(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    mission_id: &str,
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
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

    // KC準拠: 探索後スタミナは0
    while player.card_stamina.len() < player.owned_cards.len() {
        player.card_stamina.push(120);
    }
    for &i in &m.card_indices {
        if i < player.card_stamina.len() {
            player.card_stamina[i] = 0;
        }
    }

    let tidx = get_territory_index(&state.territories, &m.territory_id);
    let territory_level = tidx
        .map(|i| state.territories[i].level)
        .unwrap_or(1);
    let territory_label = tidx
        .map(|i| state.territories[i].name.clone())
        .unwrap_or_else(|| m.territory_id.clone());

    // 派遣魔獣の平均ステータス
    let (avg_stamina_before, avg_level) = {
        let n = m.card_indices.len().max(1) as u32;
        let mut sum_st = 0u32;
        let mut sum_lv = 0u32;
        for &i in &m.card_indices {
            // スタミナは0化済みなので、探索「前」の値を推定（STAMINA_EXPLORATION以上消費済み）
            sum_st = sum_st.saturating_add(STAMINA_EXPLORATION);
            let lv = player.card_levels.get(i).copied().unwrap_or(1);
            sum_lv = sum_lv.saturating_add(lv);
        }
        (sum_st / n, sum_lv / n)
    };

    let outcome = determine_exploration_outcome(
        avg_stamina_before,
        avg_level,
        territory_level,
    );

    match outcome {
        ExplorationOutcome::Failure => {
            // KC: 3〜5時間ダウン
            let rest_duration_ms = 3 * 60 * 1000 + rand::thread_rng().gen_range(0..2 * 60 * 1000);
            while player.card_rest_until.len() < player.owned_cards.len() {
                player.card_rest_until.push(0);
            }
            for &i in &m.card_indices {
                if i < player.card_rest_until.len() {
                    player.card_rest_until[i] = now.saturating_add(rest_duration_ms);
                }
            }
            push_log(
                log,
                format!(
                    "{} の探索は失敗。魔獣が力尽き、休息中となった。",
                    territory_label
                ),
            );
        }
        ExplorationOutcome::Alive => {
            // 無事帰還、たまに1〜3時間ダウン
            if rand::random::<f32>() < 0.2 {
                let rest_duration_ms =
                    1 * 60 * 1000 + rand::thread_rng().gen_range(0..2 * 60 * 1000);
                while player.card_rest_until.len() < player.owned_cards.len() {
                    player.card_rest_until.push(0);
                }
                for &i in &m.card_indices {
                    if i < player.card_rest_until.len() {
                        player.card_rest_until[i] = now.saturating_add(rest_duration_ms);
                    }
                }
                push_log(
                    log,
                    format!("{} の探索から無事帰還。疲労で少し休息中。", territory_label),
                );
            } else {
                push_log(log, format!("{} の探索から無事帰還した。", territory_label));
            }
            player.exploration_score = player
                .exploration_score
                .saturating_add(5 + territory_level as u64);
        }
        ExplorationOutcome::Succeed => {
            let bonus_mul = (territory_level as u64).max(1);
            let food = 60u64.saturating_mul(bonus_mul);
            let wood = 40u64.saturating_mul(bonus_mul);
            let stone = 25u64.saturating_mul(bonus_mul);
            let iron = 15u64.saturating_mul(bonus_mul);
            let dp = 5u64.saturating_mul(bonus_mul);
            player.resources.food = player.resources.food.saturating_add(food);
            player.resources.wood = player.resources.wood.saturating_add(wood);
            player.resources.stone = player.resources.stone.saturating_add(stone);
            player.resources.iron = player.resources.iron.saturating_add(iron);
            player.dungeon_points = player.dungeon_points.saturating_add(dp);
            player.exploration_score = player
                .exploration_score
                .saturating_add(15 + territory_level as u64 * 3);
            push_log(
                log,
                format!(
                    "{} の探索に成功！食料+{}・木+{}・石+{}・鉄+{}・DP+{}",
                    territory_label, food, wood, stone, iron, dp
                ),
            );
            // XP加算
            while player.card_exp.len() < player.owned_cards.len() {
                player.card_exp.push(0);
            }
            while player.card_levels.len() < player.owned_cards.len() {
                player.card_levels.push(1);
            }
            while player.card_status_points.len() < player.owned_cards.len() {
                player.card_status_points.push(0);
            }
            for &i in &m.card_indices {
                if i >= player.card_exp.len() {
                    continue;
                }
                let xp = 30 * territory_level as u64;
                player.card_exp[i] = player.card_exp[i].saturating_add(xp);
                let name = crate::cards::get_card(player.owned_cards[i])
                    .map(|c| c.name.to_string())
                    .unwrap_or_else(|| format!("魔獣#{}", player.owned_cards[i]));
                let awakened = *player.card_levels.get(i).unwrap_or(&1) > 99;
                let mut lvl = player.card_levels[i];
                let mut exp = player.card_exp[i];
                let mut sp = player.card_status_points[i];
                crate::model::process_level_up(
                    &mut lvl, &mut exp, &mut sp, awakened, &name, log,
                );
                player.card_levels[i] = lvl;
                player.card_exp[i] = exp;
                player.card_status_points[i] = sp;
            }
        }
        ExplorationOutcome::Excellent => {
            let bonus_mul = (territory_level as u64).max(1) * 2;
            let food = 120u64.saturating_mul(bonus_mul);
            let wood = 80u64.saturating_mul(bonus_mul);
            let stone = 50u64.saturating_mul(bonus_mul);
            let iron = 30u64.saturating_mul(bonus_mul);
            let dp = 15u64.saturating_mul(bonus_mul);
            let cp = 1u64;
            player.resources.food = player.resources.food.saturating_add(food);
            player.resources.wood = player.resources.wood.saturating_add(wood);
            player.resources.stone = player.resources.stone.saturating_add(stone);
            player.resources.iron = player.resources.iron.saturating_add(iron);
            player.dungeon_points = player.dungeon_points.saturating_add(dp);
            player.charge_points = player.charge_points.saturating_add(cp);
            player.exploration_score = player
                .exploration_score
                .saturating_add(40 + territory_level as u64 * 5);
            push_log(
                log,
                format!(
                    "★EXCELLENT！{} の探索で大量報酬獲得（食料+{}・木+{}・石+{}・鉄+{}・DP+{}・CP+{}）",
                    territory_label, food, wood, stone, iron, dp, cp
                ),
            );
            // KC: EXCELLENT でステータスポイント+1
            while player.card_status_points.len() < player.owned_cards.len() {
                player.card_status_points.push(0);
            }
            while player.card_exp.len() < player.owned_cards.len() {
                player.card_exp.push(0);
            }
            while player.card_levels.len() < player.owned_cards.len() {
                player.card_levels.push(1);
            }
            for &i in &m.card_indices {
                if i < player.card_status_points.len() {
                    player.card_status_points[i] =
                        player.card_status_points[i].saturating_add(1);
                }
                if i < player.card_exp.len() {
                    let xp = 60 * territory_level as u64;
                    player.card_exp[i] = player.card_exp[i].saturating_add(xp);
                    let name = crate::cards::get_card(player.owned_cards[i])
                        .map(|c| c.name.to_string())
                        .unwrap_or_else(|| format!("魔獣#{}", player.owned_cards[i]));
                    let awakened = *player.card_levels.get(i).unwrap_or(&1) > 99;
                    let mut lvl = player.card_levels[i];
                    let mut exp = player.card_exp[i];
                    let mut sp = player.card_status_points[i];
                    crate::model::process_level_up(
                        &mut lvl, &mut exp, &mut sp, awakened, &name, log,
                    );
                    player.card_levels[i] = lvl;
                    player.card_exp[i] = exp;
                    player.card_status_points[i] = sp;
                }
            }
        }
    }

    // 探索レベルの更新（KC: 100刻み相当で段階UP、同時派遣数が連動）
    while player.exploration_score >= 100 && player.exploration_level < 120 {
        player.exploration_score -= 100;
        player.exploration_level += 1;
        let lv = player.exploration_level;
        let slots = exploration_max_slots(lv);
        push_log(
            log,
            format!(
                "探索経験が溜まり、探索レベルが {} に上昇！同時派遣数{}体。",
                lv, slots
            ),
        );
    }

    build_game_state(state, state.turn, state.territories.clone(), log.clone(), players)
}
