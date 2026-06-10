mod alliance;
mod cards;
mod combat;
mod exploration;
mod facility;
mod market;

use std::collections::HashSet;

use rand::seq::SliceRandom;
use rand::Rng;

use alliance::*;
use cards::*;
use combat::*;
use exploration::*;
use facility::*;
use market::*;

use crate::model::{
    attack_base_owner_ids,
    build_game_state,
    can_receive_reinforcement,
    default_now_ms,
    ensure_card_monster_counts,
    generate_neutral_enemies,
    get_territory_index,
    home_territory_id,
    initial_card_monster_counts_for_owned,
    is_attackable_target,
    territories_are_adjacent,
    is_home_territory,
    parse_territory_coords,
    push_log,
    sync_home_territory_body_counts_from_player,
    territory_name,
    wave_count_for_level,
    Action,
    CardStats,
    ExplorationMission,
    FacilityPosition,
    GameState,
    MarketItemType,
    MarketListing,
};
use crate::skills::{
    apply_attack_skills,
    apply_battle_start_skills,
    apply_effect_to_character,
    check_death_skills,
    CombatCharacter,
    SkillData,
};

/// KC準拠: 攻撃1回あたりのスタミナ消費（MAX=120）
pub(crate) const STAMINA_ATTACK_FOR_XP: u32 = 25;

pub(crate) fn apply_action(
    state: &GameState,
    actor_player_id: &str,
    action: &Action,
    dev_auto_win: bool,
) -> GameState {
    let mut log = state.log.clone();
    let mut result = match action {
        Action::EndTurn => apply_end_turn_action(state, &mut log),
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
        Action::StartExploration { territory_id, card_indices } => {
            apply_start_exploration(state, &mut log, actor_player_id, territory_id, card_indices)
        }
        Action::CollectExploration { mission_id } => apply_collect_exploration(state, &mut log, actor_player_id, mission_id),
        Action::DonateAlliance { food, wood, stone, iron } => {
            apply_donate_alliance(state, &mut log, actor_player_id, *food, *wood, *stone, *iron)
        }
        Action::ProduceMonsters { card_index, amount } => {
            apply_produce_monsters(state, &mut log, actor_player_id, *card_index, *amount)
        }
    };
    // ハンドラーが state.clone() で早期リターンしても push_log の内容が失われないよう、
    // 常に外側で管理している log を最終状態に反映する
    result.log = log;
    result
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
