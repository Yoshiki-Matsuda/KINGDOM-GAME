use super::*;

pub fn select_attack_from_territory(
    state: &GameState,
    ai_id: &str,
    target_id: &str,
) -> Option<String> {
    let owned: Vec<String> = state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(ai_id))
        .map(|t| t.id.clone())
        .collect();
    owned
        .into_iter()
        .filter(|id| territories_are_adjacent(id, target_id))
        .max_by_key(|id| border_priority(state, id, target_id))
}

pub(crate) fn border_priority(state: &GameState, from_id: &str, target_id: &str) -> i32 {
    let target_coords = parse_territory_coords(target_id).unwrap_or((0, 0));
    let from_coords = parse_territory_coords(from_id).unwrap_or((0, 0));
    let dist = (from_coords.0 - target_coords.0).abs() + (from_coords.1 - target_coords.1).abs();
    let troops = state
        .territories
        .iter()
        .find(|t| t.id == from_id)
        .map(|t| t.troops as i32)
        .unwrap_or(0);
    troops * 10 - dist
}

/// 攻撃候補をスコア降順で返す（クールダウン・直前標的の回避・ランダム性あり）
pub fn rank_attack_targets(
    state: &GameState,
    ai_id: &str,
    personality: AiPersonality,
    owner_id: &str,
) -> Vec<String> {
    let now = crate::model::default_now_ms();
    let player = state.players.get(ai_id);
    let last_target = player.and_then(|p| p.ai_last_attack_target.clone());

    let owned: HashSet<String> = state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(ai_id))
        .map(|t| t.id.clone())
        .collect();

    let mut candidates: Vec<String> = Vec::new();
    for t in &state.territories {
        if t.ruin.is_some() {
            continue;
        }
        if !owned.iter().any(|oid| territories_are_adjacent(oid, &t.id)) {
            continue;
        }
        match t.owner_id.as_deref() {
            None => candidates.push(t.id.clone()),
            Some(o) if o == owner_id => candidates.push(t.id.clone()),
            Some(o) if is_ai_player_id(o) && o != ai_id => candidates.push(t.id.clone()),
            _ => {}
        }
    }
    if candidates.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(String, i32)> = candidates
        .into_iter()
        .map(|id| {
            let score = score_attack_target(
                state,
                &id,
                personality,
                owner_id,
                last_target.as_deref(),
            );
            (id, score)
        })
        .collect();

    let cooled_ok: Vec<(String, i32)> = scored
        .iter()
        .filter(|(id, _)| {
            !player
                .map(|p| target_on_cooldown(p, id, now))
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    if !cooled_ok.is_empty() {
        scored = cooled_ok;
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored.into_iter().map(|(id, _)| id).collect()
}

pub(crate) fn score_attack_target(
    state: &GameState,
    target_id: &str,
    personality: AiPersonality,
    owner_id: &str,
    avoid_target: Option<&str>,
) -> i32 {
    let territory = state
        .territories
        .iter()
        .find(|t| t.id == target_id);
    let Some(territory) = territory else {
        return i32::MIN;
    };

    let mut score = territory.level as i32 * 10;
    if territory.owner_id.as_deref() == Some(owner_id) {
        score += match personality {
            AiPersonality::Aggressive => 30,
            AiPersonality::Balanced => 20,
            AiPersonality::Defensive => 8,
        };
    }
    if avoid_target == Some(target_id) {
        score -= 200;
    }
    let (_, def_counts, _) = resolve_territory_defenders(territory, &state.players);
    let def_power: i32 = def_counts
        .iter()
        .map(|c| (*c as i32).max(1) * 4)
        .sum();
    score -= def_power / 3;
    score += rand::random::<i32>().abs() % 18;
    score
}

pub(crate) fn target_on_cooldown(player: &crate::model::PlayerData, target_id: &str, now: u64) -> bool {
    player
        .ai_attack_cooldowns
        .iter()
        .any(|(id, until)| id == target_id && *until > now)
}

pub(crate) fn prune_ai_attack_cooldowns(player: &mut crate::model::PlayerData, now: u64) {
    player.ai_attack_cooldowns.retain(|(_, until)| *until > now);
}

pub fn is_ai_recovering(player: &crate::model::PlayerData, now: u64) -> bool {
    player.ai_recover_until > now
}

pub fn record_ai_attack_outcome(
    player: &mut crate::model::PlayerData,
    target_id: &str,
    conquered: bool,
    now: u64,
) {
    if !is_ai_player_id(&player.player_id) {
        return;
    }
    prune_ai_attack_cooldowns(player, now);
    if conquered {
        player
            .ai_attack_cooldowns
            .retain(|(id, _)| id != target_id);
        if player.ai_recover_until <= now {
            player.ai_recover_until = 0;
        }
    } else {
        player.ai_last_attack_target = Some(target_id.to_string());
        if let Some(entry) = player
            .ai_attack_cooldowns
            .iter_mut()
            .find(|(id, _)| id == target_id)
        {
            entry.1 = entry.1.max(now + AI_ATTACK_COOLDOWN_MS);
        } else {
            player
                .ai_attack_cooldowns
                .push((target_id.to_string(), now + AI_ATTACK_COOLDOWN_MS));
        }
        player.ai_recover_until = player.ai_recover_until.max(now + AI_RECOVERY_MS);
    }
}

pub(crate) fn has_active_attack_march(state: &GameState, ai_id: &str) -> bool {
    state
        .players
        .get(ai_id)
        .map(|p| p.marches.iter().any(|m| m.kind == MarchKind::Attack))
        .unwrap_or(false)
}

pub(crate) fn formation_viable_for_target(
    state: &GameState,
    formation: &AttackFormation,
    player: &crate::model::PlayerData,
    target: &Territory,
) -> bool {
    let (_, def_counts, def_names) = resolve_territory_defenders(target, &state.players);
    let score = score_formation(
        &formation.card_indices,
        player,
        &def_names,
        &def_counts,
    );
    let def_power: i64 = def_counts
        .iter()
        .map(|c| (*c as i64).max(1) * 6)
        .sum::<i64>()
        .max(12);
    score * 100 >= def_power * 35
}
