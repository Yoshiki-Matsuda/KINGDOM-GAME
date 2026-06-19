mod alliance;
mod cards;
mod card_stats;
mod combat;
mod facility;
mod formation;
mod march;
mod market;

use std::collections::HashSet;

use crate::server_mode::ServerMode;

use rand::seq::SliceRandom;
use rand::Rng;

use alliance::*;
use cards::*;
use card_stats::*;
use combat::*;
use facility::*;
use formation::*;
use march::*;
use market::*;

use crate::model::push_system_event;

use crate::model::{
    attack_base_owner_ids,
    build_game_state,
    can_receive_reinforcement,
    default_now_ms,
    ensure_card_monster_counts,
    generate_neutral_enemies,
    generate_neutral_enemies_for_territory,
    resolve_territory_defenders,
    get_territory_index,
    home_territory_id,
    initial_card_monster_counts_for_owned,
    is_attackable_target,
    territories_are_adjacent,
    is_home_territory,
    parse_territory_coords,
    sync_home_territory_body_counts_from_player,
    territory_name,
    wave_count_for_level,
    Action,
    CardStatBonuses,
    CardStats,
    FacilityPosition,
    GameEvent,
    GameState,
    MarketItemType,
    MarketListing,
    StoredFormedUnit,
};
use crate::skills::{
    apply_attack_skills,
    apply_battle_start_skills,
    apply_effect_to_character,
    check_death_skills,
    CombatCharacter,
    SkillData,
};

/// KC準拠: 攻撃1回あたりのスタミナ消費（出発時・`StartMarch` で使用）
pub(crate) const STAMINA_ATTACK: u32 = crate::config::DEFAULT_STAMINA_ATTACK;

pub use march::{exploration_max_slots, tick_marches};

pub(crate) fn apply_action(
    state: &GameState,
    actor_player_id: &str,
    action: &Action,
    dev_auto_win: bool,
    server_mode: ServerMode,
) -> GameState {
    if server_mode == ServerMode::Pve {
        if let Some(msg) = validate_pve_action(state, actor_player_id, action) {
            let mut log = state.log.clone();
            push_system_event(&mut log, &msg);
            return build_game_state(
                state,
                state.territories.clone(),
                log,
                state.players.clone(),
            );
        }
    }

    let mut log = state.log.clone();
    let mut result = match action {
        Action::Deploy {
            territory_id,
            count,
            monsters_per_body,
            body_names,
        } => apply_deploy_action(state, &mut log, actor_player_id, territory_id, *count, monsters_per_body, body_names),
        Action::Attack {
            from_territory_id,
            to_territory_id,
            count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
        } => apply_attack_action(
            state,
            &mut log,
            actor_player_id,
            from_territory_id,
            to_territory_id,
            *count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
            dev_auto_win,
            false,
        ),
        Action::BuildBase { territory_id } => apply_build_base(state, &mut log, actor_player_id, territory_id),
        Action::BuildFacility { facility_id, level, position } => {
            apply_build_facility(state, &mut log, actor_player_id, facility_id, *level, position)
        }
        Action::SynthesizeCard { base_card_index, material_card_indices } => {
            apply_synthesize_card(state, &mut log, actor_player_id, *base_card_index, material_card_indices)
        }
        Action::CreateAlliance { name } => apply_create_alliance(state, &mut log, actor_player_id, name),
        Action::JoinAlliance { alliance_id } => apply_join_alliance(state, &mut log, actor_player_id, alliance_id),
        Action::LeaveAlliance => apply_leave_alliance(state, &mut log, actor_player_id),
        Action::ListOnFleaMarket { item, price } => apply_list_on_flea_market(state, &mut log, actor_player_id, item, *price),
        Action::BuyFromFleaMarket { listing_id } => apply_buy_from_flea_market(state, &mut log, actor_player_id, listing_id),
        Action::CancelFleaMarketListing { listing_id } => apply_cancel_flea_market_listing(state, &mut log, actor_player_id, listing_id),
        Action::StartMarch {
            kind,
            from_territory_id,
            to_territory_id,
            count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
            formed_unit_id,
        } => apply_start_march(
            state,
            &mut log,
            actor_player_id,
            dev_auto_win,
            *kind,
            from_territory_id,
            to_territory_id,
            *count,
            monsters_per_body,
            body_names,
            unit_name,
            speed_per_body,
            skills_per_body,
            stats_per_body,
            owned_card_indices,
            formed_unit_id,
        ),
        Action::DonateAlliance { food, wood, stone, iron } => {
            apply_donate_alliance(state, &mut log, actor_player_id, *food, *wood, *stone, *iron)
        }
        Action::ProduceMonsters { card_index, amount } => {
            apply_produce_monsters(state, &mut log, actor_player_id, *card_index, *amount)
        }
        Action::SetFormedUnits { units } => {
            apply_set_formed_units(state, &mut log, actor_player_id, units)
        }
        Action::AllocateCardStats {
            card_index,
            speed,
            attack,
            intelligence,
            defense,
            magic_defense,
        } => apply_allocate_card_stats(
            state,
            &mut log,
            actor_player_id,
            *card_index,
            CardStatBonuses {
                speed: *speed,
                attack: *attack,
                intelligence: *intelligence,
                defense: *defense,
                magic_defense: *magic_defense,
            },
        ),
    };
    // ハンドラーが state.clone() で早期リターンしても push_log の内容が失われないよう、
    // 常に外側で管理している log を最終状態に反映する
    result.log = log;
    result
}

fn validate_pve_action(state: &GameState, actor: &str, action: &Action) -> Option<String> {
    use crate::pve_world::{is_ai_player_id, is_human_player_id};

    if is_human_player_id(actor) {
        if state.world_owner_id.as_deref() != Some(actor) {
            return Some("このワールドの所有者ではありません。".to_string());
        }
    }

    match action {
        Action::CreateAlliance { .. }
        | Action::JoinAlliance { .. }
        | Action::LeaveAlliance
        | Action::DonateAlliance { .. } => {
            return Some("PVEでは同盟機能は利用できません。".to_string());
        }
        Action::ListOnFleaMarket { .. } if is_ai_player_id(actor) => {
            return Some("AI勢力はフリマに出品できません。".to_string());
        }
        Action::Attack { to_territory_id, .. } if is_human_player_id(actor) => {
            let target_owner = state
                .territories
                .iter()
                .find(|t| t.id == *to_territory_id)
                .and_then(|t| t.owner_id.clone());
            if let Some(owner) = target_owner {
                if is_human_player_id(&owner) && owner != actor {
                    return Some("PVEでは他プレイヤーへの攻撃はできません。".to_string());
                }
            }
        }
        Action::Deploy { territory_id, .. } if is_human_player_id(actor) => {
            let owner = state
                .territories
                .iter()
                .find(|t| t.id == *territory_id)
                .and_then(|t| t.owner_id.clone());
            if let Some(owner) = owner {
                if is_human_player_id(&owner) && owner != actor {
                    return Some("PVEでは他プレイヤー領への援軍はできません。".to_string());
                }
            }
        }
        _ => {}
    }
    None
}

/// owned_cards と同じ並びの Vec から、削除インデックス（昇順）に合わせて要素を除去
fn remove_indices_from_parallel_vec<T: Clone>( vec: &mut Vec<T>, sorted_asc: &[usize]) {
    for &idx in sorted_asc.iter().rev() {
        if idx < vec.len() {
            vec.remove(idx);
        }
    }
}

fn consume_inventory_costs(
    inventory: &mut Vec<crate::items::InventoryItem>,
    costs: &[(&str, u32)],
) -> bool {
    let has_all = costs.iter().all(|(item_id, count)| {
        inventory
            .iter()
            .find(|item| item.item_id == *item_id)
            .map(|item| item.count >= *count)
            .unwrap_or(false)
    });
    if !has_all {
        return false;
    }
    for (item_id, count) in costs {
        if let Some(item) = inventory.iter_mut().find(|item| item.item_id == *item_id) {
            item.count = item.count.saturating_sub(*count);
        }
    }
    inventory.retain(|item| item.count > 0);
    true
}
