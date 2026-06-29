mod formation;
mod combat;
mod exploration;

pub(crate) use formation::*;
pub(crate) use combat::*;
pub(crate) use exploration::*;
use std::collections::HashSet;

use crate::cards::{get_card, get_card_skills, CardStats};
use crate::config;
use crate::model::{
    apply_action, default_now_ms, formation_owned_slots_in_slot_order,
    is_kc_unit_ready_to_deploy, march_busy_formed_unit_ids, march_locked_card_slots,
    parse_territory_coords, resolve_territory_defenders, territories_are_adjacent, Action,
    AiPersonality, CardStatBonuses, GameState, MarchKind, PlayerData, StoredFormedUnit,
    Territory,
};
use crate::model_actions::{exploration_max_slots, STAMINA_ATTACK};
use crate::pve_world::is_ai_player_id;
use crate::server_mode::ServerMode;
use crate::skills::SkillData;

const STAMINA_EXPLORE: u32 = config::DEFAULT_STAMINA_EXPLORATION;
/// 攻撃遠征のため最低限キープする食料（全消費すると占領ループが止まる）
const AI_FOOD_RESERVE: u64 = 150;
/// 攻撃失敗後に生産へ振り向くときの食料キープ量
const AI_RECOVERY_FOOD_RESERVE: u64 = 50;
/// 同一領地への再攻撃クールダウン（ms）
const AI_ATTACK_COOLDOWN_MS: u64 = 5 * 60 * 1000;
/// 攻撃失敗後に攻撃を控えて内製に集中する時間（ms）
const AI_RECOVERY_MS: u64 = 3 * 60 * 1000;
/// `model_actions/cards.rs` の `FOOD_PER_MONSTER_PRODUCE` と同期
const FOOD_PER_MONSTER_PRODUCE: u64 = 2;

fn formation_has_march_stamina(
    player: &PlayerData,
    card_indices: &[usize],
    kind: MarchKind,
) -> bool {
    let cost = match kind {
        MarchKind::Attack => STAMINA_ATTACK,
        MarchKind::Explore => STAMINA_EXPLORE,
        _ => return true,
    };
    for &i in card_indices {
        let st = player
            .card_stamina
            .get(i)
            .copied()
            .unwrap_or(config::max_card_stamina());
        if st < cost {
            return false;
        }
    }
    true
}

/// 資源取得優先の施設（有効な facility_id のみ）
const AI_FACILITY_PRIORITIES: &[&str] = &[
    "field",
    "lumber_mill",
    "quarry",
    "ironworks",
    "warehouse",
    "stronghold",
];

/// AI ターン処理のサーバーログ用サマリー（ゲーム内ログとは別）
#[derive(Debug, Default, Clone)]
pub struct AiTurnReport {
    pub facility_built: Option<String>,
    pub monsters_produced: u32,
    pub explorations_collected: u32,
    pub exploration_started: Option<String>,
    pub attack: Option<(String, String)>,
    pub market_purchase: bool,
    pub stats_allocated_slots: u32,
}

impl AiTurnReport {
    pub fn summarize(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(facility_id) = &self.facility_built {
            parts.push(format!("施設={facility_id}"));
        }
        if self.monsters_produced > 0 {
            parts.push(format!("魔獣生産+{}", self.monsters_produced));
        }
        if self.explorations_collected > 0 {
            parts.push(format!("探索回収×{}", self.explorations_collected));
        }
        if let Some(territory_id) = &self.exploration_started {
            parts.push(format!("探索派遣→{territory_id}"));
        }
        if let Some((from, to)) = &self.attack {
            parts.push(format!("攻撃 {from}→{to}"));
        }
        if self.market_purchase {
            parts.push("市場購入".to_string());
        }
        if self.stats_allocated_slots > 0 {
            parts.push(format!("育成×{}", self.stats_allocated_slots));
        }
        if parts.is_empty() {
            "待機".to_string()
        } else {
            parts.join(" ")
        }
    }
}


pub fn run_ai_faction_turn(
    state: &GameState,
    ai_id: &str,
    personality: AiPersonality,
    owner_id: &str,
    dev_auto_win: bool,
) -> (GameState, AiTurnReport) {
    let mut report = AiTurnReport::default();
    let mut current = state.clone();
    crate::model::tick_world(&mut current, dev_auto_win, ServerMode::Pve);
    let now = crate::model::default_now_ms();
    if let Some(player) = current.players.get_mut(ai_id) {
        ensure_ai_formed_units(player, now);
    }
    let recovering = current
        .players
        .get(ai_id)
        .map(|p| is_ai_recovering(p, now))
        .unwrap_or(false);

    // 1. 探索派遣（空きスロットがあれば資源収集のため継続派遣）
    run_exploration_dispatches(&mut current, ai_id, dev_auto_win, &mut report);

    // 2. 施設建設（人間と同じ素材消費・建設キュー）
    if let Some(facility_id) = current
        .players
        .get(ai_id)
        .and_then(next_facility_to_build)
    {
        let before = current.players.get(ai_id).map(|p| p.facilities.clone());
        let next = apply_action(
            &current,
            ai_id,
            &Action::BuildFacility {
                facility_id: facility_id.clone(),
                level: 1,
                position: Some(crate::model::FacilityPosition { col: 2, row: 2 }),
            },
            dev_auto_win,
            ServerMode::Pve,
        );
        if facility_build_succeeded(before.as_deref(), next.players.get(ai_id)) {
            report.facility_built = Some(facility_id);
        }
        current = next;
    }

    // 4. 魔獣生産（攻撃失敗後は食料キープを下げて増産）
    let food_reserve = if recovering {
        AI_RECOVERY_FOOD_RESERVE
    } else {
        AI_FOOD_RESERVE
    };
    report.monsters_produced =
        run_monster_production(&mut current, ai_id, dev_auto_win, food_reserve);
    if recovering {
        report.monsters_produced = report.monsters_produced.saturating_add(
            run_monster_production(&mut current, ai_id, dev_auto_win, AI_RECOVERY_FOOD_RESERVE / 2),
        );
    }

    // 4b. 魔獣育成（未配分ステータスポイントをバランス型で一括振り分け）
    report.stats_allocated_slots =
        run_ai_stat_allocation(&mut current, ai_id, dev_auto_win);

    // 5. 攻撃遠征（進行中の攻撃遠征・回復中は送らない）
    if !has_active_attack_march(&current, ai_id) && !recovering {
        let ranked_targets = rank_attack_targets(&current, ai_id, personality, owner_id);
        for target_id in ranked_targets {
            let target = current.territories.iter().find(|t| t.id == target_id).cloned();
            if let (Some(target), Some(from_id)) = (
                target,
                select_attack_from_territory(&current, ai_id, &target_id),
            ) {
                let Some(formation) = pick_ai_attack_from_formed_units(&current, ai_id, &target, now)
                    .map(|(id, f)| (Some(id), f))
                    .or_else(|| {
                        build_attack_formation(&current, ai_id, &target).map(|f| (None, f))
                    })
                else {
                    continue;
                };
                let (formed_unit_id, formation) = formation;
                let Some(player) = current.players.get(ai_id) else {
                    break;
                };
                if !formation_viable_for_target(&current, &formation, player, &target) {
                    continue;
                }
                if !formation_has_march_stamina(player, &formation.card_indices, MarchKind::Attack) {
                    continue;
                }
                let before_marches = player.marches.len();
                let count = formation.card_indices.len() as u32;
                let mut next = apply_action(
                    &current,
                    ai_id,
                    &Action::StartMarch {
                        kind: MarchKind::Attack,
                        from_territory_id: from_id.clone(),
                        to_territory_id: target_id.clone(),
                        count,
                        monsters_per_body: Some(formation.monster_counts),
                        body_names: Some(formation.body_names),
                        unit_name: Some(format!("{ai_id}遠征")),
                        speed_per_body: Some(formation.speeds),
                        skills_per_body: Some(formation.skills),
                        stats_per_body: Some(formation.stats),
                        owned_card_indices: Some(formation.card_indices),
                        formed_unit_id,
                    },
                    dev_auto_win,
                    ServerMode::Pve,
                );
                let after_marches = next
                    .players
                    .get(ai_id)
                    .map(|p| p.marches.len())
                    .unwrap_or(0);
                if after_marches > before_marches {
                    if let Some(p) = next.players.get_mut(ai_id) {
                        p.ai_last_attack_target = Some(target_id.clone());
                    }
                    report.attack = Some((from_id, target_id));
                    current = next;
                    break;
                }
            }
        }
    }

    report.market_purchase = try_ai_market_purchase(&mut current, ai_id, dev_auto_win);
    (current, report)
}


fn facility_build_succeeded(
    before: Option<&[crate::model::BuiltFacility]>,
    after_player: Option<&crate::model::PlayerData>,
) -> bool {
    let (Some(before), Some(after)) = (before, after_player) else {
        return false;
    };
    if after.facilities.len() > before.len() {
        return true;
    }
    after.facilities.iter().any(|f_after| {
        before
            .iter()
            .find(|f| f.facility_id == f_after.facility_id)
            .map(|f_before| {
                f_after.level > f_before.level
                    || f_after.build_complete_at != f_before.build_complete_at
            })
            .unwrap_or(true)
    })
}

/// 未配分ptを5ステ均等配分（余りは speed→attack→intelligence→defense→magic_defense の順に+1）
fn balanced_stat_delta(unspent: u32) -> CardStatBonuses {
    if unspent == 0 {
        return CardStatBonuses::default();
    }
    let base = unspent / 5;
    let remainder = unspent % 5;
    let mut values = [base; 5];
    for v in values.iter_mut().take(remainder as usize) {
        *v += 1;
    }
    CardStatBonuses {
        speed: values[0],
        attack: values[1],
        intelligence: values[2],
        defense: values[3],
        magic_defense: values[4],
    }
}

fn run_ai_stat_allocation(state: &mut GameState, ai_id: &str, dev_auto_win: bool) -> u32 {
    let card_count = state
        .players
        .get(ai_id)
        .map(|p| p.owned_cards.len())
        .unwrap_or(0);
    let mut slots_allocated = 0u32;
    for card_index in 0..card_count {
        let unspent = state
            .players
            .get(ai_id)
            .and_then(|p| p.card_status_points.get(card_index))
            .copied()
            .unwrap_or(0);
        if unspent == 0 {
            continue;
        }
        let delta = balanced_stat_delta(unspent);
        let before_points = unspent;
        *state = apply_action(
            state,
            ai_id,
            &Action::AllocateCardStats {
                card_index,
                speed: delta.speed,
                attack: delta.attack,
                intelligence: delta.intelligence,
                defense: delta.defense,
                magic_defense: delta.magic_defense,
            },
            dev_auto_win,
            ServerMode::Pve,
        );
        let after_points = state
            .players
            .get(ai_id)
            .and_then(|p| p.card_status_points.get(card_index))
            .copied()
            .unwrap_or(before_points);
        if after_points < before_points {
            slots_allocated += 1;
        }
    }
    slots_allocated
}

fn run_monster_production(
    state: &mut GameState,
    ai_id: &str,
    dev_auto_win: bool,
    food_reserve: u64,
) -> u32 {
    let card_count = state
        .players
        .get(ai_id)
        .map(|p| p.owned_cards.len())
        .unwrap_or(0);
    let mut total_produced = 0u32;
    for card_index in 0..card_count {
        let (food, room) = {
            let Some(player) = state.players.get(ai_id) else {
                break;
            };
            let cap = crate::model::MAX_MONSTER_COUNT_PER_CARD_SLOT;
            let cur = player
                .card_monster_counts
                .get(card_index)
                .copied()
                .unwrap_or(1)
                .min(cap);
            let room = cap.saturating_sub(cur);
            (player.resources.food, room)
        };
        if room == 0 {
            continue;
        }
        let affordable = (food.saturating_sub(food_reserve) / FOOD_PER_MONSTER_PRODUCE)
            .min(room as u64) as u32;
        if affordable == 0 {
            break;
        }
        let amount = affordable.min(25);
        let before_count = state
            .players
            .get(ai_id)
            .and_then(|p| p.card_monster_counts.get(card_index))
            .copied()
            .unwrap_or(1);
        *state = apply_action(
            state,
            ai_id,
            &Action::ProduceMonsters { card_index, amount },
            dev_auto_win,
            ServerMode::Pve,
        );
        let after_count = state
            .players
            .get(ai_id)
            .and_then(|p| p.card_monster_counts.get(card_index))
            .copied()
            .unwrap_or(before_count);
        total_produced = total_produced.saturating_add(after_count.saturating_sub(before_count));
    }
    total_produced
}

fn next_facility_to_build(player: &crate::model::PlayerData) -> Option<String> {
    for id in AI_FACILITY_PRIORITIES {
        if !player.facilities.iter().any(|f| f.facility_id == *id) {
            return Some(id.to_string());
        }
    }
    None
}

fn try_ai_market_purchase(state: &mut GameState, ai_id: &str, dev_auto_win: bool) -> bool {
    let listings: Vec<String> = state
        .market_listings
        .iter()
        .filter(|l| crate::pve_world::is_human_player_id(&l.seller_id))
        .map(|l| l.listing_id.clone())
        .collect();
    let gold = state.players.get(ai_id).map(|p| p.resources.gold).unwrap_or(0);
    for listing_id in listings {
        let price = state
            .market_listings
            .iter()
            .find(|l| l.listing_id == listing_id)
            .map(|l| l.price)
            .unwrap_or(u64::MAX);
        if price > gold / 2 {
            continue;
        }
        let action = Action::BuyFromFleaMarket { listing_id };
        *state = apply_action(state, ai_id, &action, dev_auto_win, ServerMode::Pve);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AiPersonality, Territory, WorldConfig};
    use crate::pve_world::new_pve;

    #[test]
    fn ai_monster_production_consumes_food_like_human_action() {
        let state = new_pve("tester", WorldConfig::default());
        let ai_id = "ai_faction_0";
        let food_before = state.players.get(ai_id).unwrap().resources.food;
        let (after, report) = run_ai_faction_turn(&state, ai_id, AiPersonality::Balanced, "tester", false);
        let food_after = after.players.get(ai_id).unwrap().resources.food;
        if report.monsters_produced > 0 {
            let expected_min_cost = report.monsters_produced as u64 * FOOD_PER_MONSTER_PRODUCE;
            assert!(
                food_before.saturating_sub(food_after) >= expected_min_cost,
                "produced {} monsters should cost at least {} food (before={food_before} after={food_after})",
                report.monsters_produced,
                expected_min_cost
            );
        }
    }

    #[test]
    fn ai_facility_priority_starts_with_field() {
        let state = new_pve("tester", WorldConfig::default());
        let player = state.players.get("ai_faction_0").unwrap();
        assert_eq!(next_facility_to_build(player).as_deref(), Some("field"));
    }

    #[test]
    fn formation_respects_unit_cost_cap() {
        let state = new_pve("tester", WorldConfig::default());
        let ai_id = "ai_faction_0";
        let target = Territory {
            id: "c_10_10".to_string(),
            name: "平原".to_string(),
            level: 1,
            owner_id: None,
            troops: 0,
            body_monster_counts: None,
            body_names: None,
            ruin: None,
            is_base: false,
            durability: 0,
            max_durability: 0,
            tower_level: 0,
        };
        let formation = build_attack_formation(&state, ai_id, &target).expect("formation");
        assert!(!formation.card_indices.is_empty());
        assert!(formation.card_indices.len() <= 3);
        let player = state.players.get(ai_id).unwrap();
        let total: f32 = formation
            .card_indices
            .iter()
            .map(|&i| card_effective_cost(player, i))
            .sum();
        assert!(total <= unit_cost_cap_for(&state, ai_id) + 0.001);
        assert_eq!(formation.stats.len(), formation.card_indices.len());
        assert_eq!(formation.monster_counts.len(), formation.card_indices.len());
    }

    #[test]
    fn lv1_neutral_enemy_count_is_modest() {
        let (_, counts, _) = crate::model::generate_neutral_enemies_for_territory(1, "c_1_1");
        assert_eq!(counts, vec![18]);
    }

    #[test]
    fn ai_attack_failure_enters_recovery_and_cooldown() {
        let mut player = crate::model::PlayerData::new(
            "ai_faction_0".to_string(),
            "c_0_0".to_string(),
        );
        let now = 1_000_000u64;
        record_ai_attack_outcome(&mut player, "c_1_0", false, now);
        assert!(is_ai_recovering(&player, now + 1));
        assert!(player
            .ai_attack_cooldowns
            .iter()
            .any(|(id, until)| id == "c_1_0" && *until > now));
        assert_eq!(player.ai_last_attack_target.as_deref(), Some("c_1_0"));
    }

    #[test]
    fn rank_attack_targets_deprioritizes_last_target() {
        let mut state = new_pve("tester", WorldConfig::default());
        let ai_id = "ai_faction_0";
        let home = state.players.get(ai_id).unwrap().home_territory_id.clone();
        let neighbors: Vec<String> = state
            .territories
            .iter()
            .filter(|t| {
                t.owner_id.is_none()
                    && crate::model::territories_are_adjacent(&home, &t.id)
            })
            .map(|t| t.id.clone())
            .collect();
        if neighbors.len() < 2 {
            return;
        }
        if let Some(player) = state.players.get_mut(ai_id) {
            player.ai_last_attack_target = Some(neighbors[0].clone());
        }
        let ranked = rank_attack_targets(&state, ai_id, AiPersonality::Balanced, "tester");
        assert!(!ranked.is_empty());
        if ranked.len() >= 2 {
            assert_ne!(ranked[0], neighbors[0]);
        }
    }

    #[test]
    fn ai_picks_explorable_territory_without_troops() {
        use crate::model::territories_are_adjacent;

        let mut state = new_pve("tester", WorldConfig::default());
        let ai_id = "ai_faction_0";
        let home = state.players.get(ai_id).unwrap().home_territory_id.clone();
        let Some(neighbor_id) = state
            .territories
            .iter()
            .find(|t| t.owner_id.is_none() && territories_are_adjacent(&home, &t.id))
            .map(|t| t.id.clone())
        else {
            return;
        };
        let idx = state
            .territories
            .iter()
            .position(|t| t.id == neighbor_id)
            .unwrap();
        state.territories[idx].owner_id = Some(ai_id.to_string());
        state.territories[idx].is_base = false;
        state.territories[idx].troops = 0;

        let target = pick_exploration_territory(&state, ai_id);
        assert_eq!(target.as_deref(), Some(neighbor_id.as_str()));
        assert_eq!(
            select_explore_from_territory(&state, ai_id, &neighbor_id).as_deref(),
            Some(home.as_str())
        );
    }

    #[test]
    fn conquest_bonus_within_level_range() {
        for _ in 0..30 {
            let (f, w, s, i) = crate::model::conquest_resource_bonus(1);
            assert!((15..=80).contains(&f));
            assert!((15..=80).contains(&w));
            assert!((15..=80).contains(&s));
            assert!((15..=80).contains(&i));
        }
        let (f3, _, _, _) = crate::model::conquest_resource_bonus(3);
        assert!((35..=180).contains(&f3));
    }

    #[test]
    fn balanced_stat_delta_splits_evenly() {
        let d = balanced_stat_delta(10);
        assert_eq!(d.speed, 2);
        assert_eq!(d.attack, 2);
        assert_eq!(d.intelligence, 2);
        assert_eq!(d.defense, 2);
        assert_eq!(d.magic_defense, 2);
        assert_eq!(d.total(), 10);

        let d7 = balanced_stat_delta(7);
        assert_eq!(d7.total(), 7);
        assert_eq!(d7.speed, 2);
        assert_eq!(d7.attack, 2);
        assert_eq!(d7.intelligence, 1);
        assert_eq!(d7.defense, 1);
        assert_eq!(d7.magic_defense, 1);
    }

    #[test]
    fn run_ai_stat_allocation_spends_unspent_points() {
        let mut state = new_pve("tester", WorldConfig::default());
        let ai_id = "ai_faction_0";
        if let Some(player) = state.players.get_mut(ai_id) {
            let n = player.owned_cards.len();
            player.card_status_points = vec![10; n];
            crate::model::ensure_card_stat_bonuses(player);
        }
        let slots = run_ai_stat_allocation(&mut state, ai_id, false);
        let player = state.players.get(ai_id).unwrap();
        assert_eq!(slots, player.owned_cards.len() as u32);
        assert!(player.card_status_points.iter().all(|&p| p == 0));
        assert_eq!(player.card_stat_bonuses[0].speed, 2);
        assert_eq!(player.card_stat_bonuses[0].attack, 2);
        assert_eq!(player.card_stat_bonuses[0].intelligence, 2);
        assert_eq!(player.card_stat_bonuses[0].defense, 2);
        assert_eq!(player.card_stat_bonuses[0].magic_defense, 2);
    }
}
