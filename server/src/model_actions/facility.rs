use super::*;
use crate::model::push_actor_system_event;

const BUILD_TIME_FAST_5: [u64; 5] = [60, 180, 600, 1800, 3600];
const BUILD_TIME_STORAGE_3: [u64; 3] = [60, 300, 900];
const BUILD_TIME_STANDARD_3: [u64; 3] = [120, 600, 1800];
const BUILD_TIME_STANDARD_4: [u64; 4] = [120, 600, 1800, 3600];

fn facility_build_time_seconds(facility_id: &str, level: u8) -> Option<u64> {
    let level_index = usize::from(level.checked_sub(1)?);
    let times: &[u64] = match facility_id {
        "field"
        | "lumber_mill"
        | "ironworks"
        | "quarry"
        | "trading_post"
        | "fortress"
        | "training_tower"
        | "monster_barracks" => &BUILD_TIME_FAST_5,
        "warehouse" => &BUILD_TIME_STORAGE_3,
        "stronghold"
        | "beast_lab"
        | "demihuman_lab"
        | "spirit_lab"
        | "undead_lab"
        | "giant_lab"
        | "demon_lab"
        | "dragon_lab"
        | "library"
        | "hero_statue"
        | "guardian_shrine"
        | "war_god_shrine" => &BUILD_TIME_STANDARD_3,
        "battle_lab" => &BUILD_TIME_STANDARD_4,
        _ => return None,
    };
    times.get(level_index).copied()
}

fn facility_build_costs(level: u8) -> Option<Vec<(&'static str, u32)>> {
    let costs = match level {
        1 => vec![("rotten_wood", 20), ("ancient_stone", 15)],
        2 => vec![("rotten_wood", 50), ("ancient_stone", 30), ("rusty_gear", 10)],
        3 => vec![("rotten_wood", 100), ("refined_iron", 20), ("mystic_crystal", 10)],
        4 => vec![("rotten_wood", 200), ("refined_iron", 50), ("shining_magicstone", 10)],
        5 => vec![("rotten_wood", 400), ("shining_magicstone", 30), ("guardian_core", 3)],
        _ => return None,
    };
    Some(costs)
}

pub(super) fn apply_build_base(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    territory_id: &str,
) -> GameState {
    let mut territories = state.territories.clone();
    let idx = match get_territory_index(&territories, territory_id) {
        Some(i) => i,
        None => return state.clone(),
    };
    if territories[idx].owner_id.as_deref() != Some(actor_player_id) {
        return state.clone();
    }
    if territories[idx].is_base || is_home_territory(territory_id) {
        return state.clone();
    }
    let mut players = state.players.clone();
    if let Some(player) = players.get_mut(actor_player_id) {
        let cost_food = 200u64;
        let cost_wood = 300u64;
        let cost_stone = 200u64;
        let cost_iron = 100u64;
        if player.resources.food < cost_food
            || player.resources.wood < cost_wood
            || player.resources.stone < cost_stone
            || player.resources.iron < cost_iron
        {
            push_actor_system_event(log, actor_player_id, "資源が足りません。");
            return state.clone();
        }
        player.resources.food -= cost_food;
        player.resources.wood -= cost_wood;
        player.resources.stone -= cost_stone;
        player.resources.iron -= cost_iron;
    }
    territories[idx].is_base = true;
    let lvl = territories[idx].level as u32;
    territories[idx].max_durability = 400u32.saturating_add(lvl.saturating_mul(120));
    territories[idx].durability = territories[idx].max_durability;
    territories[idx].tower_level = 1;
    let name = territory_name(&territories, territory_id).to_string();
    push_actor_system_event(log, actor_player_id, &format!("{}に前線基地を建設しました！", name));

    build_game_state(state, territories, log.clone(), players)
}

/// KC準拠: 施設建設/レベルアップ（拠点ごとに同時1件のみ建設可能）
pub(super) fn apply_build_facility(
    state: &GameState,
    log: &mut Vec<GameEvent>,
    actor_player_id: &str,
    facility_id: &str,
    level: u8,
    position: &Option<FacilityPosition>,
) -> GameState {
    let now = default_now_ms();
    let Some(build_seconds) = facility_build_time_seconds(facility_id, level) else {
        push_actor_system_event(log, actor_player_id, "施設IDまたはレベル指定が不正です。");
        return state.clone();
    };
    let Some(costs) = facility_build_costs(level) else {
        push_actor_system_event(log, actor_player_id, "レベル指定が不正です。");
        return state.clone();
    };

    let mut players = state.players.clone();
    let Some(player) = players.get_mut(actor_player_id) else {
        return state.clone();
    };
    if !consume_inventory_costs(&mut player.inventory, &costs) {
        push_actor_system_event(log, actor_player_id, "施設建設に必要な素材が足りません。");
        return state.clone();
    }

    // 既に同名施設があるか確認し、レベルアップか新規かを判定
    let mut facilities = player.facilities.clone();

    // キュー制限: 拠点内で未完了の施設があるなら新規建設/LvUPを拒否
    let has_building = facilities
        .iter()
        .any(|f| f.build_complete_at.map(|t| t > now).unwrap_or(false));
    if has_building {
        push_actor_system_event(log, actor_player_id, "既に建設中の施設があります（同時1件まで）。");
        return state.clone();
    }

    let build_complete_at = now.saturating_add(build_seconds.saturating_mul(1000));

    match facilities.iter_mut().find(|f| f.facility_id == facility_id) {
        Some(existing) => {
            if level <= existing.level {
                push_actor_system_event(log, actor_player_id, "現在より高いレベルを指定してください。");
                return state.clone();
            }
            if let Some(requested_position) = position {
                if let Some(existing_position) = existing.position {
                    if existing_position != *requested_position {
                        push_actor_system_event(log, actor_player_id, "施設の配置座標が一致しません。");
                        return state.clone();
                    }
                } else {
                    existing.position = Some(*requested_position);
                }
            }
            existing.level = level;
            existing.build_complete_at = Some(build_complete_at);
            push_actor_system_event(log, actor_player_id, &format!("施設「{}」をLv{}へアップグレード開始。", facility_id, level));
        }
        None => {
            let Some(position) = *position else {
                push_actor_system_event(log, actor_player_id, "施設の配置座標を指定してください。");
                return state.clone();
            };
            facilities.push(crate::model::BuiltFacility {
                facility_id: facility_id.to_string(),
                level,
                build_complete_at: Some(build_complete_at),
                position: Some(position),
            });
            push_actor_system_event(log, actor_player_id, &format!("施設「{}」の建設を開始。", facility_id));
        }
    }

    player.facilities = facilities;

    build_game_state(state, state.territories.clone(), log.clone(), players)
}
