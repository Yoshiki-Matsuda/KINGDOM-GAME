use super::*;

pub(crate) fn ensure_stamina_vec(player: &mut crate::model::PlayerData) {
    let max_stamina = config::max_card_stamina();
    while player.card_stamina.len() < player.owned_cards.len() {
        player.card_stamina.push(max_stamina);
    }
}

pub(crate) fn consume_stamina_for_march(
    actor_player_id: &str,
    player: &mut crate::model::PlayerData,
    card_indices: &[usize],
    cost: u32,
    log: &mut Vec<GameEvent>,
    dev_auto_win: bool,
) -> bool {
    ensure_stamina_vec(player);
    let now = default_now_ms();
    let mut used = HashSet::new();
    for &i in card_indices {
        if !used.insert(i) {
            push_actor_system_event(log, actor_player_id, "同じ魔獣を重複指定できません。");
            return false;
        }
        if i >= player.owned_cards.len() {
            push_actor_system_event(log, actor_player_id, "無効な魔獣スロットです。");
            return false;
        }
        let rest_until = player.card_rest_until.get(i).copied().unwrap_or(0);
        if rest_until > now {
            push_actor_system_event(log, actor_player_id, "休息中の魔獣を派遣できません。");
            return false;
        }
    }
    if dev_auto_win && crate::pve_world::is_human_player_id(actor_player_id) {
        return true;
    }
    for &i in card_indices {
        let st = player.card_stamina.get(i).copied().unwrap_or(config::max_card_stamina());
        if st < cost {
            push_actor_system_event(log, actor_player_id, "スタミナが足りない魔獣が含まれています。");
            return false;
        }
    }
    for &i in card_indices {
        player.card_stamina[i] = player.card_stamina[i].saturating_sub(cost);
    }
    true
}

pub(crate) fn apply_start_march(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    dev_auto_win: bool,
    kind: MarchKind,
    from_territory_id: &str,
    to_territory_id: &str,
    count: u32,
    monsters_per_body: &Option<Vec<u32>>,
    body_names: &Option<Vec<String>>,
    unit_name: &Option<String>,
    speed_per_body: &Option<Vec<u32>>,
    skills_per_body: &Option<Vec<SkillData>>,
    stats_per_body: &Option<Vec<CardStats>>,
    owned_card_indices: &Option<Vec<usize>>,
    formed_unit_id: &Option<String>,
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };

    if count == 0 {
        push_actor_system_event(log, actor_player_id, "出撃する体がありません。");
        return state.clone();
    }

    let from_idx = get_territory_index(&state.territories, from_territory_id);
    let to_idx = get_territory_index(&state.territories, to_territory_id);
    if from_idx.is_none() || to_idx.is_none() {
        push_actor_system_event(log, actor_player_id, "無効な領地です。");
        return state.clone();
    }

    if !territories_are_adjacent(from_territory_id, to_territory_id) {
        push_actor_system_event(log, actor_player_id, &format!("{}と{}は隣接していません。", from_territory_id, to_territory_id));
        return state.clone();
    }

    match kind {
        MarchKind::Attack => {
            if is_home_territory(to_territory_id) {
                push_actor_system_event(log, actor_player_id, "本拠地は攻撃できません。");
                return state.clone();
            }
            let base_owners = attack_base_owner_ids(state, actor_player_id);
            if !is_attackable_target(&state.territories, to_territory_id, &base_owners) {
                push_actor_system_event(log, actor_player_id, "攻撃できない領地です。");
                return state.clone();
            }
            let from_owner = state.territories[from_idx.unwrap()]
                .owner_id
                .as_deref();
            if from_owner != Some(actor_player_id) {
                push_actor_system_event(log, actor_player_id, "自領からのみ攻撃できます。");
                return state.clone();
            }
            let oci = match owned_card_indices {
                Some(v) if v.len() == count as usize => v,
                _ => {
                    push_actor_system_event(log, actor_player_id, "攻撃には編成スロット情報が必要です。");
                    return state.clone();
                }
            };
            if !validate_march_dispatch(player, actor_player_id, count, owned_card_indices, formed_unit_id, log) {
                return state.clone();
            }
            if !consume_stamina_for_march(
                actor_player_id,
                player,
                oci,
                config::stamina_attack_cost(),
                log,
                dev_auto_win,
            ) {
                return state.clone();
            }
        }
        MarchKind::Explore => {
            let to_owner = state.territories[to_idx.unwrap()]
                .owner_id
                .as_deref();
            if to_owner != Some(actor_player_id) {
                push_actor_system_event(log, actor_player_id, "自領地のみ探索できます。");
                return state.clone();
            }
            if to_territory_id == player.home_territory_id.as_str() {
                push_actor_system_event(log, actor_player_id, "本拠地からは探索を派遣しません。");
                return state.clone();
            }
            if state.territories[to_idx.unwrap()].is_base {
                push_actor_system_event(log, actor_player_id, "拠点や塔からは探索を派遣できません。");
                return state.clone();
            }
            let oci = match owned_card_indices {
                Some(v) if !v.is_empty() && v.len() == count as usize => v,
                _ => {
                    push_actor_system_event(log, actor_player_id, "探索に使用する魔獣を選んでください。");
                    return state.clone();
                }
            };
            let max_slots = exploration_max_slots(player.exploration_level);
            if oci.len() > max_slots {
                push_actor_system_event(log, actor_player_id, &format!(
                        "同時派遣数が探索レベル({}体まで)を超えています。",
                        max_slots
                    ));
                return state.clone();
            }
            let now = default_now_ms();
            let active_explore_bodies = active_explore_bodies_in_flight(player, now);
            if active_explore_bodies + oci.len() > max_slots {
                push_actor_system_event(log, actor_player_id, "これ以上探索を出せません。");
                return state.clone();
            }
            if !validate_march_dispatch(player, actor_player_id, count, owned_card_indices, formed_unit_id, log) {
                return state.clone();
            }
            if !consume_stamina_for_march(
                actor_player_id,
                player,
                oci,
                config::stamina_exploration_cost(),
                log,
                dev_auto_win,
            ) {
                return state.clone();
            }
        }
        MarchKind::Deploy => {
            if is_home_territory(to_territory_id) {
                push_actor_system_event(log, actor_player_id, "本拠地へ援軍は不要です。");
                return state.clone();
            }
            let allied = player.allied_player_ids.clone();
            if !can_receive_reinforcement(&state.territories, actor_player_id, &allied, to_territory_id)
            {
                push_actor_system_event(log, actor_player_id, "援軍を送れない領地です。");
                return state.clone();
            }
            if !validate_march_dispatch(player, actor_player_id, count, owned_card_indices, formed_unit_id, log) {
                return state.clone();
            }
        }
        MarchKind::Return => {
            push_actor_system_event(log, actor_player_id, "帰還は自動生成されます。");
            return state.clone();
        }
    }

    let resolved_stats = owned_card_indices
        .as_ref()
        .filter(|indices| indices.len() == count as usize)
        .map(|indices| {
            crate::model::resolve_authoritative_body_stats(
                player,
                indices,
                monsters_per_body.as_deref(),
            )
        });
    let resolved_speeds = resolved_stats
        .as_ref()
        .map(|stats| stats.iter().map(|s| s.speed).collect::<Vec<_>>());
    let avg_speed = crate::model::average_speed(resolved_speeds.as_deref(), count);
    let travel_ms = crate::model::travel_time_ms(from_territory_id, to_territory_id, avg_speed);
    let now = default_now_ms();
    let march_id = format!("march-{}", now);
    let to_name = territory_name(&state.territories, to_territory_id).to_string();

    let label = match kind {
        MarchKind::Attack => "攻撃遠征",
        MarchKind::Deploy => "援軍",
        MarchKind::Explore => "探索",
        MarchKind::Return => "帰還",
    };
    player.marches.push(MarchMission {
        march_id: march_id.clone(),
        kind,
        from_territory_id: from_territory_id.to_string(),
        to_territory_id: to_territory_id.to_string(),
        started_at: now,
        arrives_at: now.saturating_add(travel_ms),
        count,
        monsters_per_body: monsters_per_body.clone(),
        body_names: body_names.clone(),
        unit_name: unit_name.clone(),
        speed_per_body: resolved_speeds.or_else(|| speed_per_body.clone()),
        skills_per_body: skills_per_body.clone(),
        stats_per_body: resolved_stats.or_else(|| stats_per_body.clone()),
        owned_card_indices: owned_card_indices.clone(),
        formed_unit_id: formed_unit_id.clone(),
    });

    let dispatch_msg = match kind {
        MarchKind::Explore => {
            push_explore_dispatch_event(log, actor_player_id, &to_name, &format!("探索を{}へ派遣しました。", to_name));
            None
        }
        _ => Some(format!(
            "{}を{}へ派遣しました（{}・到着まで約{}秒）。",
            label, to_name, label, travel_ms / 1000
        )),
    };
    if let Some(msg) = dispatch_msg {
        push_actor_system_event(log, actor_player_id, &msg);
    }

    build_game_state(state, state.territories.clone(), log.clone(), players)
}
