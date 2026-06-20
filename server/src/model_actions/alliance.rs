use super::*;
use crate::model::{push_actor_system_event, push_alliance_event};

pub(super) fn apply_create_alliance(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    name: &str,
) -> GameState {
    if state.alliances.iter().any(|a| a.member_ids.contains(&actor_player_id.to_string())) {
        push_actor_system_event(log, actor_player_id, "既に同盟に所属しています。");
        return state.clone();
    }
    let alliance_id = format!("alliance_{}", state.alliances.len() + 1);
    let alliance = crate::model::Alliance {
        id: alliance_id.clone(),
        name: name.to_string(),
        leader_id: actor_player_id.to_string(),
        member_ids: vec![actor_player_id.to_string()],
        territory_points: 0,
        level: 1,
        donated_total: 0,
        parent_alliance_id: None,
        child_alliance_ids: vec![],
    };
    let mut new_state = state.clone();
    new_state.alliances.push(alliance);
    push_alliance_event(log, &format!("同盟「{}」を結成しました！", name));
    new_state.log = log.clone();
    new_state
}

pub(super) fn apply_join_alliance(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    alliance_id: &str,
) -> GameState {
    if state.alliances.iter().any(|a| a.member_ids.contains(&actor_player_id.to_string())) {
        push_actor_system_event(log, actor_player_id, "既に同盟に所属しています。");
        return state.clone();
    }
    let mut new_state = state.clone();
    if let Some(alliance) = new_state.alliances.iter_mut().find(|a| a.id == alliance_id) {
        alliance.member_ids.push(actor_player_id.to_string());
        push_alliance_event(log, &format!("同盟「{}」に参加しました！", alliance.name));
    } else {
        push_actor_system_event(log, actor_player_id, "同盟が見つかりません。");
    }
    new_state.log = log.clone();
    new_state
}

pub(super) fn apply_leave_alliance(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
) -> GameState {
    let mut new_state = state.clone();
    let player_id = actor_player_id.to_string();
    if let Some(alliance) = new_state.alliances.iter_mut().find(|a| a.member_ids.contains(&player_id)) {
        alliance.member_ids.retain(|id| id != &player_id);
        push_alliance_event(log, &format!("同盟「{}」を脱退しました。", alliance.name));
        if alliance.member_ids.is_empty() {
            let alliance_name = alliance.name.clone();
            new_state.alliances.retain(|a| !a.member_ids.is_empty());
            push_alliance_event(log, &format!("同盟「{}」は解散しました。", alliance_name));
        } else if alliance.leader_id == player_id {
            alliance.leader_id = alliance.member_ids[0].clone();
            push_alliance_event(log, &format!("リーダーが{}に引き継がれました。", alliance.leader_id));
        }
    }
    new_state.log = log.clone();
    new_state
}

/// KC仕様準拠: 本拠地陥落により、被攻撃側プレイヤーの所属同盟を
/// 攻撃側プレイヤーの所属同盟の配下同盟にする（既存の配下は親同盟の直下に移る）。
pub(super) fn subjugate_alliance_of(
    victim_player_id: &str,
    victor_player_id: &str,
    alliances: &mut [crate::model::Alliance],
    log: &mut Vec<GameEvent>,
) {
    let victim_ix = alliances
        .iter()
        .position(|a| a.member_ids.iter().any(|m| m == victim_player_id));
    let victor_ix = alliances
        .iter()
        .position(|a| a.member_ids.iter().any(|m| m == victor_player_id));
    let (vi, ai) = match (victim_ix, victor_ix) {
        (Some(vi), Some(ai)) if vi != ai => (vi, ai),
        _ => return,
    };
    // 既に同じ親同盟の直下なら何もしない
    if alliances[vi].parent_alliance_id.as_deref() == Some(&alliances[ai].id) {
        return;
    }
    // 被害側の既存の配下同盟は、攻撃側の配下に付け替え
    let old_children = alliances[vi].child_alliance_ids.clone();
    for child_id in &old_children {
        if let Some(cidx) = alliances.iter().position(|a| &a.id == child_id) {
            alliances[cidx].parent_alliance_id = Some(alliances[ai].id.clone());
            let new_id = alliances[cidx].id.clone();
            if !alliances[ai].child_alliance_ids.contains(&new_id) {
                alliances[ai].child_alliance_ids.push(new_id);
            }
        }
    }
    alliances[vi].child_alliance_ids.clear();

    // 被害側→攻撃側の親子付け替え
    let victim_id = alliances[vi].id.clone();
    alliances[vi].parent_alliance_id = Some(alliances[ai].id.clone());
    if !alliances[ai].child_alliance_ids.contains(&victim_id) {
        alliances[ai].child_alliance_ids.push(victim_id.clone());
    }
    let victor_name = alliances[ai].name.clone();
    let victim_name = alliances[vi].name.clone();
    push_alliance_event(log, &format!("同盟「{}」は同盟「{}」の配下同盟となった！", victim_name, victor_name));
}

/// KC仕様準拠: 同盟レベルごとの累積寄付閾値
/// Lv1→Lv2: 10,000、Lv2→Lv3: 26,800、… 最大Lv15
pub(crate) const ALLIANCE_LEVEL_THRESHOLDS: &[u64] = &[
    0,           // Lv1（初期）
    10_000,      // Lv2
    36_800,      // Lv3
    91_600,      // Lv4
    210_400,     // Lv5
    455_200,     // Lv6
    940_000,     // Lv7
    1_924_800,   // Lv8
    3_951_600,   // Lv9
    8_134_400,   // Lv10
    16_747_200,  // Lv11
    34_374_000,  // Lv12
    70_250_800,  // Lv13
    142_947_600, // Lv14
    289_559_400, // Lv15
];

pub(crate) fn alliance_level_from_donation(donated: u64) -> u32 {
    let mut lv = 1u32;
    for (i, &thr) in ALLIANCE_LEVEL_THRESHOLDS.iter().enumerate() {
        if donated >= thr {
            lv = (i as u32) + 1;
        }
    }
    lv.min(15).max(1)
}

pub(super) fn apply_donate_alliance(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    food: u64,
    wood: u64,
    stone: u64,
    iron: u64,
) -> GameState {
    let total = food.saturating_add(wood).saturating_add(stone).saturating_add(iron);
    if total == 0 {
        return state.clone();
    }
    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };
    if player.resources.food < food
        || player.resources.wood < wood
        || player.resources.stone < stone
        || player.resources.iron < iron
    {
        push_actor_system_event(log, actor_player_id, "寄付する資源が足りません。");
        return state.clone();
    }
    let mut alliances = state.alliances.clone();
    let Some(ai) = alliances
        .iter()
        .position(|a| a.member_ids.iter().any(|m| m == actor_player_id))
    else {
        push_actor_system_event(log, actor_player_id, "同盟に所属していないため寄付できません。");
        return state.clone();
    };
    player.resources.food -= food;
    player.resources.wood -= wood;
    player.resources.stone -= stone;
    player.resources.iron -= iron;
    alliances[ai].donated_total = alliances[ai].donated_total.saturating_add(total);
    let donated = alliances[ai].donated_total;
    let new_level = alliance_level_from_donation(donated);
    if new_level > alliances[ai].level {
        push_alliance_event(log, &format!("同盟への寄付が実を結び、同盟レベルが {} になった！", new_level));
    }
    alliances[ai].level = new_level;
    let mut out = build_game_state(state, state.territories.clone(), log.clone(), players);
    out.alliances = alliances;
    out
}
