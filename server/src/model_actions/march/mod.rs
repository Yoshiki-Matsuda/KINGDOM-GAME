mod dispatch;

pub(super) use dispatch::*;
use std::collections::HashSet;

use crate::config;
use crate::server_mode::ServerMode;

use super::*;

use crate::model::{MarchKind, MarchMission, push_actor_system_event, push_explore_complete_event, push_explore_dispatch_event, push_explore_level_up_event};

/// KC準拠: 探索でスロットあたり加算される同時派遣数の閾値
pub fn exploration_max_slots(exploration_level: u32) -> usize {
    match exploration_level {
        0..=19 => 1,
        20..=39 => 2,
        40..=59 => 3,
        60..=79 => 4,
        80..=99 => 5,
        _ => 6,
    }
}

/// 未到着の探索遠征で派遣中の体数（同時派遣数の判定用）
fn active_explore_bodies_in_flight(player: &crate::model::PlayerData, now: u64) -> usize {
    player
        .marches
        .iter()
        .filter(|m| m.kind == MarchKind::Explore && m.arrives_at > now)
        .map(|m| {
            m.owned_card_indices
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(m.count as usize)
        })
        .sum()
}

/// 進行中遠征（未到着）で使用中の魔獣スロット
pub fn march_locked_card_slots(
    player: &crate::model::PlayerData,
    now: u64,
) -> HashSet<usize> {
    crate::model::march_locked_card_slots(player, now)
}

/// 進行中遠征で本拠から派遣中の体数（到着済み除く・帰還中含む）
pub(crate) fn march_bodies_away_count(player: &crate::model::PlayerData, now: u64) -> u32 {
    player
        .marches
        .iter()
        .filter(|m| m.arrives_at > now && m.owned_card_indices.is_some())
        .map(|m| m.count)
        .sum()
}

fn march_busy_formed_unit_ids(
    player: &crate::model::PlayerData,
    now: u64,
) -> HashSet<String> {
    crate::model::march_busy_formed_unit_ids(player, now)
}

fn validate_march_dispatch(
    player: &crate::model::PlayerData,
    actor_player_id: &str,
    count: u32,
    owned_card_indices: &Option<Vec<usize>>,
    formed_unit_id: &Option<String>,
    log: &mut Vec<GameEvent>,
) -> bool {
    let now = default_now_ms();
    if let Some(ref oci) = owned_card_indices {
        if !oci.is_empty() {
            let mut used = HashSet::new();
            for &i in oci {
                if !used.insert(i) {
                    push_actor_system_event(log, actor_player_id, "同じ魔獣を重複指定できません。");
                    return false;
                }
                if i >= player.owned_cards.len() {
                    push_actor_system_event(log, actor_player_id, "無効な魔獣スロットです。");
                    return false;
                }
            }
            let locked = march_locked_card_slots(player, now);
            for &i in oci {
                if locked.contains(&i) {
                    push_actor_system_event(log, actor_player_id, "遠征中の魔獣を派遣できません。");
                    return false;
                }
            }
            let away = march_bodies_away_count(player, now);
            let cap = player.owned_cards.len() as u32;
            if away.saturating_add(count) > cap {
                push_actor_system_event(log, actor_player_id, &format!(
                        "本拠に残っている体数が足りません（遠征中{}体・今回{}体・上限{}体）。",
                        away, count, cap
                    ));
                return false;
            }
        }
    }
    if let Some(uid) = formed_unit_id {
        if march_busy_formed_unit_ids(player, now).contains(uid) {
            push_actor_system_event(log, actor_player_id, "この編成は既に遠征中です。");
            return false;
        }
    }
    true
}


pub fn apply_explore_arrival(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    march: &MarchMission,
) -> GameState {
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };

    let tidx = get_territory_index(&state.territories, &march.to_territory_id);
    let territory_level = tidx
        .map(|i| state.territories[i].level)
        .unwrap_or(1);
    let territory_label = tidx
        .map(|i| state.territories[i].name.clone())
        .unwrap_or_else(|| march.to_territory_id.clone());

    let (food, wood, stone, iron) = crate::model::exploration_resource_bonus(territory_level);
    let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
    let res_cap = 10_000u64.saturating_add(bonuses.storage_capacity as u64 * 150);

    player.resources.food = (player.resources.food.saturating_add(food)).min(res_cap);
    player.resources.wood = (player.resources.wood.saturating_add(wood)).min(res_cap);
    player.resources.stone = (player.resources.stone.saturating_add(stone)).min(res_cap);
    player.resources.iron = (player.resources.iron.saturating_add(iron)).min(res_cap);

    player.exploration_score = player
        .exploration_score
        .saturating_add(5 + territory_level as u64);

    let idxs = march.owned_card_indices.clone().unwrap_or_default();
    let base_xp = 15_u64.saturating_add(territory_level as u64 * 5);
    while player.card_exp.len() < player.owned_cards.len() {
        player.card_exp.push(0);
    }
    while player.card_levels.len() < player.owned_cards.len() {
        player.card_levels.push(1);
    }
    while player.card_status_points.len() < player.owned_cards.len() {
        player.card_status_points.push(0);
    }
    for &i in &idxs {
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

    while player.exploration_score >= 100 && player.exploration_level < 120 {
        player.exploration_score -= 100;
        player.exploration_level += 1;
        let lv = player.exploration_level;
        let slots = exploration_max_slots(lv);
        push_explore_level_up_event(log, actor_player_id, &format!(
                "探索経験が溜まり、探索レベルが {} に上昇！同時派遣数{}体。",
                lv, slots
            ));
    }

    push_explore_complete_event(
        log,
        actor_player_id,
        &territory_label,
        food,
        wood,
        stone,
        iron,
    );

    build_game_state(state, state.territories.clone(), log.clone(), players)
}

fn remove_march(player: &mut crate::model::PlayerData, march_id: &str) {
    if let Some(ix) = player.marches.iter().position(|m| m.march_id == march_id) {
        player.marches.remove(ix);
    }
}

fn maybe_spawn_return_march(
    player: &mut crate::model::PlayerData,
    outbound: &MarchMission,
    conquered: bool,
) {
    if !conquered || outbound.kind != MarchKind::Attack {
        return;
    }
    let travel_ms = outbound.arrives_at.saturating_sub(outbound.started_at);
    let now = default_now_ms();
    player.marches.push(MarchMission {
        march_id: format!("march-return-{}", now),
        kind: MarchKind::Return,
        from_territory_id: outbound.to_territory_id.clone(),
        to_territory_id: outbound.from_territory_id.clone(),
        started_at: now,
        arrives_at: now.saturating_add(travel_ms),
        count: outbound.count,
        monsters_per_body: outbound.monsters_per_body.clone(),
        body_names: outbound.body_names.clone(),
        unit_name: outbound.unit_name.clone(),
        speed_per_body: outbound.speed_per_body.clone(),
        skills_per_body: None,
        stats_per_body: None,
        owned_card_indices: outbound.owned_card_indices.clone(),
        formed_unit_id: outbound.formed_unit_id.clone(),
    });
}

pub fn tick_marches(
    state: &mut GameState,
    log: &mut Vec<GameEvent>,
    dev_auto_win: bool,
    server_mode: ServerMode,
    include_returns: bool,
) -> bool {
    let now = default_now_ms();
    let mut due: Vec<(String, MarchMission)> = Vec::new();
    for (player_id, player) in &state.players {
        for march in &player.marches {
            if march.arrives_at <= now && (include_returns || march.kind != MarchKind::Return) {
                due.push((player_id.clone(), march.clone()));
            }
        }
    }
    if due.is_empty() {
        return false;
    }
    due.sort_by_key(|(_, m)| m.arrives_at);

    let mut changed = false;
    for (player_id, march) in due {
        if let Some(player) = state.players.get(&player_id) {
            if !player.marches.iter().any(|m| m.march_id == march.march_id) {
                continue;
            }
        } else {
            continue;
        }

        if let Some(player) = state.players.get_mut(&player_id) {
            remove_march(player, &march.march_id);
        }
        changed = true;

        match march.kind {
            MarchKind::Return => {
                push_actor_system_event(log, &player_id, &format!(
                        "{}が帰還しました。",
                        march.unit_name.as_deref().unwrap_or("遠征隊")
                    ));
            }
            MarchKind::Attack => {
                let to_id = march.to_territory_id.clone();
                let owner_before = state
                    .territories
                    .iter()
                    .find(|t| t.id == to_id)
                    .and_then(|t| t.owner_id.clone());
                let next = apply_attack_action(
                    state,
                    log,
                    &player_id,
                    &march.from_territory_id,
                    &march.to_territory_id,
                    march.count,
                    &march.monsters_per_body,
                    &march.body_names,
                    &march.unit_name,
                    &march.speed_per_body,
                    &march.skills_per_body,
                    &march.stats_per_body,
                    &march.owned_card_indices,
                    dev_auto_win,
                    true,
                );
                *state = next;
                let conquered = state
                    .territories
                    .iter()
                    .find(|t| t.id == to_id)
                    .and_then(|t| t.owner_id.as_deref())
                    == Some(player_id.as_str())
                    && owner_before.as_deref() != Some(player_id.as_str());
                if crate::pve_world::is_ai_player_id(&player_id) {
                    if let Some(player) = state.players.get_mut(&player_id) {
                        crate::ai_actions::record_ai_attack_outcome(
                            player,
                            &to_id,
                            conquered,
                            default_now_ms(),
                        );
                    }
                }
                if let Some(player) = state.players.get_mut(&player_id) {
                    maybe_spawn_return_march(player, &march, conquered);
                }
            }
            MarchKind::Deploy => {
                let next = apply_deploy_action(
                    state,
                    log,
                    &player_id,
                    &march.to_territory_id,
                    march.count,
                    &march.monsters_per_body,
                    &march.body_names,
                );
                *state = next;
                if let Some(unit_id) = &march.formed_unit_id {
                    if let Some(player) = state.players.get_mut(&player_id) {
                        player.formed_units.retain(|u| u.id != *unit_id);
                    }
                }
            }
            MarchKind::Explore => {
                let next = apply_explore_arrival(state, log, &player_id, &march);
                *state = next;
            }
        }
    }
    let _ = server_mode;
    changed
}

#[cfg(test)]
mod march_validation_tests {
    use super::*;
    use crate::model::{apply_action, Action, GameState, MarchMission, DEFAULT_PLAYER_ID};
    use crate::server_mode::ServerMode;

    fn adjacent_target(state: &GameState) -> String {
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let (col, row) = crate::model::parse_territory_coords(&home).unwrap();
        format!("c_{}_{}", col + 1, row)
    }

    fn start_march_attack(state: &GameState, oci: Vec<usize>, count: u32) -> GameState {
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let to = adjacent_target(state);
        apply_action(
            state,
            DEFAULT_PLAYER_ID,
            &Action::StartMarch {
                kind: MarchKind::Attack,
                from_territory_id: home,
                to_territory_id: to,
                count,
                monsters_per_body: Some(vec![10; count as usize]),
                body_names: Some((0..count).map(|i| format!("B{}", i)).collect()),
                unit_name: Some("隊".into()),
                speed_per_body: Some(vec![5; count as usize]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(oci),
                formed_unit_id: None,
            },
            false,
            ServerMode::Pve,
        )
    }

    fn push_active_march(state: &mut GameState, march: MarchMission) {
        state
            .players
            .get_mut(DEFAULT_PLAYER_ID)
            .unwrap()
            .marches
            .push(march);
    }

    fn pve_player_state() -> GameState {
        let mut state = GameState::default();
        state.world_owner_id = Some(DEFAULT_PLAYER_ID.to_string());
        state
    }

    #[test]
    fn start_march_rejects_duplicate_slots_on_active_march() {
        let mut state = pve_player_state();
        let now = default_now_ms();
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let to = adjacent_target(&state);
        push_active_march(
            &mut state,
            MarchMission {
                march_id: "existing".into(),
                kind: MarchKind::Attack,
                from_territory_id: home,
                to_territory_id: to,
                started_at: now,
                arrives_at: u64::MAX,
                count: 2,
                monsters_per_body: Some(vec![10, 10]),
                body_names: Some(vec!["A".into(), "B".into()]),
                unit_name: Some("隊".into()),
                speed_per_body: Some(vec![5, 5]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(vec![0, 1]),
                formed_unit_id: None,
            },
        );
        let before_len = state.players.get(DEFAULT_PLAYER_ID).unwrap().marches.len();
        state = start_march_attack(&state, vec![1, 2], 2);
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.marches.len(), before_len);
        assert!(state.log.iter().any(|l| l.message.contains("遠征中の魔獣")));
    }

    #[test]
    fn march_helpers_detect_away_bodies_and_locked_slots() {
        let mut state = pve_player_state();
        let now = default_now_ms();
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let player = state.players.get_mut(DEFAULT_PLAYER_ID).unwrap();
        player.marches.push(MarchMission {
            march_id: "away".into(),
            kind: MarchKind::Attack,
            from_territory_id: home,
            to_territory_id: "c_0_0".into(),
            started_at: now,
            arrives_at: u64::MAX,
            count: 5,
            monsters_per_body: None,
            body_names: None,
            unit_name: None,
            speed_per_body: None,
            skills_per_body: None,
            stats_per_body: None,
            owned_card_indices: Some((0..5).collect()),
            formed_unit_id: None,
        });
        let player = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(march_bodies_away_count(player, now), 5);
        assert!(march_locked_card_slots(player, now).contains(&3));
        let cap = player.owned_cards.len() as u32;
        assert!(march_bodies_away_count(player, now).saturating_add(6) > cap);
    }

    #[test]
    fn start_march_rejects_while_unit_returning() {
        let mut state = pve_player_state();
        let now = default_now_ms();
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let to = adjacent_target(&state);
        push_active_march(
            &mut state,
            MarchMission {
                march_id: "returning".into(),
                kind: MarchKind::Return,
                from_territory_id: to.clone(),
                to_territory_id: home.clone(),
                started_at: now,
                arrives_at: now + 60_000,
                count: 3,
                monsters_per_body: Some(vec![10; 3]),
                body_names: Some(vec!["A".into(), "B".into(), "C".into()]),
                unit_name: Some("隊".into()),
                speed_per_body: Some(vec![5; 3]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(vec![0, 1, 2]),
                formed_unit_id: Some("unit-1".into()),
            },
        );
        let before_len = state.players.get(DEFAULT_PLAYER_ID).unwrap().marches.len();
        state = start_march_attack(&state, vec![0, 1, 2], 3);
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.marches.len(), before_len);
        assert!(
            state
                .log
                .iter()
                .any(|l| l.message.contains("遠征中の魔獣") || l.message.contains("既に遠征中"))
        );
    }

    #[test]
    fn start_march_rejects_when_home_bodies_exhausted() {
        let mut state = pve_player_state();
        let now = default_now_ms();
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let to = adjacent_target(&state);
        push_active_march(
            &mut state,
            MarchMission {
                march_id: "big".into(),
                kind: MarchKind::Attack,
                from_territory_id: home,
                to_territory_id: to,
                started_at: now,
                arrives_at: u64::MAX,
                count: 7,
                monsters_per_body: Some(vec![10; 7]),
                body_names: Some(vec!["A".into(); 7]),
                unit_name: Some("隊".into()),
                speed_per_body: Some(vec![5; 7]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some((0..7).collect()),
                formed_unit_id: None,
            },
        );
        let before_len = state.players.get(DEFAULT_PLAYER_ID).unwrap().marches.len();
        // 残り3スロットしかないのに4体派遣 → スロット重複または体数超過で拒否
        state = start_march_attack(&state, vec![7, 8, 9, 6], 4);
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.marches.len(), before_len);
        assert!(
            state.log.iter().any(|l| {
                l.message.contains("本拠に残っている体数が足りません")
                    || l.message.contains("遠征中の魔獣")
            }),
            "log={:?}",
            state.log
        );
    }

    #[test]
    fn produce_monsters_rejects_slot_on_active_march() {
        let mut state = pve_player_state();
        let now = default_now_ms();
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let to = adjacent_target(&state);
        push_active_march(
            &mut state,
            MarchMission {
                march_id: "explore".into(),
                kind: MarchKind::Explore,
                from_territory_id: home,
                to_territory_id: to,
                started_at: now,
                arrives_at: u64::MAX,
                count: 1,
                monsters_per_body: Some(vec![10]),
                body_names: Some(vec!["A".into()]),
                unit_name: Some("探索".into()),
                speed_per_body: Some(vec![5]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(vec![3]),
                formed_unit_id: None,
            },
        );
        let food_before = state.players.get(DEFAULT_PLAYER_ID).unwrap().resources.food;
        state = apply_action(
            &state,
            DEFAULT_PLAYER_ID,
            &Action::ProduceMonsters {
                card_index: 3,
                amount: 1,
            },
            false,
            ServerMode::Pve,
        );
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.resources.food, food_before);
        assert!(state.log.iter().any(|l| l.message.contains("遠征中の魔獣は生産できません")));
    }

    fn owned_explore_target(state: &mut GameState) -> String {
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let (col, row) = crate::model::parse_territory_coords(&home).unwrap();
        let to = format!("c_{}_{}", col + 1, row);
        let idx = crate::model::get_territory_index(&state.territories, &to).unwrap();
        state.territories[idx].owner_id = Some(DEFAULT_PLAYER_ID.to_string());
        state.territories[idx].is_base = false;
        to
    }

    fn start_march_explore(state: &GameState, to: &str, oci: Vec<usize>) -> GameState {
        let home = state
            .players
            .get(DEFAULT_PLAYER_ID)
            .unwrap()
            .home_territory_id
            .clone();
        let count = oci.len() as u32;
        apply_action(
            state,
            DEFAULT_PLAYER_ID,
            &Action::StartMarch {
                kind: MarchKind::Explore,
                from_territory_id: home,
                to_territory_id: to.to_string(),
                count,
                monsters_per_body: Some(vec![10; count as usize]),
                body_names: Some((0..count).map(|i| format!("B{}", i)).collect()),
                unit_name: Some("探索隊".into()),
                speed_per_body: Some(vec![5; count as usize]),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(oci),
                formed_unit_id: None,
            },
            false,
            ServerMode::Pve,
        )
    }

    #[test]
    fn start_march_explore_rejects_multi_body_at_level_one() {
        let mut state = pve_player_state();
        let to = owned_explore_target(&mut state);
        let before_len = state.players.get(DEFAULT_PLAYER_ID).unwrap().marches.len();
        state = start_march_explore(&state, &to, vec![0, 1, 2]);
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.marches.len(), before_len);
        assert!(state.log.iter().any(|l| l.message.contains("同時派遣数が探索レベル")));
    }

    #[test]
    fn start_march_explore_accepts_single_body_at_level_one() {
        let mut state = pve_player_state();
        let to = owned_explore_target(&mut state);
        let before_len = state.players.get(DEFAULT_PLAYER_ID).unwrap().marches.len();
        state = start_march_explore(&state, &to, vec![0]);
        let p = state.players.get(DEFAULT_PLAYER_ID).unwrap();
        assert_eq!(p.marches.len(), before_len + 1);
        assert!(p.marches.iter().any(|m| m.kind == MarchKind::Explore));
    }
}
