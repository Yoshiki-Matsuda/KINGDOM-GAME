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

/// 攻撃遠征の編成（dev_bot と同様に stats / 魔獣数を Action に載せる）
#[derive(Debug, Clone)]
pub struct AttackFormation {
    pub card_indices: Vec<usize>,
    pub monster_counts: Vec<u32>,
    pub body_names: Vec<String>,
    pub speeds: Vec<u32>,
    pub skills: Vec<SkillData>,
    pub stats: Vec<CardStats>,
}

/// KC 編成スロット [前衛, 中衛, リーダー] に本拠スロット列を割り当てる
fn slots_to_formed_indices(slots: &[usize]) -> [i32; 3] {
    let mut out = [-1i32; 3];
    match slots.len() {
        1 => out[2] = slots[0] as i32,
        2 => {
            out[0] = slots[0] as i32;
            out[2] = slots[1] as i32;
        }
        3 => {
            out[0] = slots[0] as i32;
            out[1] = slots[1] as i32;
            out[2] = slots[2] as i32;
        }
        _ => {}
    }
    out
}

fn ai_formed_units_valid(player: &PlayerData, max_units: usize, now: u64) -> bool {
    if player.formed_units.is_empty() || player.formed_units.len() > max_units {
        return false;
    }
    let locked = march_locked_card_slots(player, now);
    let mut used = HashSet::new();
    for unit in &player.formed_units {
        if !is_kc_unit_ready_to_deploy(&unit.indices) {
            return false;
        }
        for slot in formation_owned_slots_in_slot_order(&unit.indices) {
            if slot >= player.owned_cards.len() || locked.contains(&slot) || !used.insert(slot) {
                return false;
            }
        }
    }
    true
}

fn best_unit_combo(player: &PlayerData, candidates: &[usize], cost_cap: f32) -> Option<Vec<usize>> {
    let max_bodies = candidates.len().min(3);
    let mut best: Option<(i64, Vec<usize>)> = None;
    for size in 1..=max_bodies {
        pick_combinations(candidates, size, 0, &mut Vec::new(), &mut |combo: &[usize]| {
            let mut species = HashSet::new();
            for &idx in combo {
                if !species.insert(player.owned_cards[idx]) {
                    return;
                }
            }
            if formation_total_cost(player, combo) > cost_cap + 0.0001 {
                return;
            }
            let score: i64 = combo
                .iter()
                .map(|&i| player.card_monster_counts.get(i).copied().unwrap_or(1) as i64)
                .sum();
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, combo.to_vec()));
            }
        });
    }
    best.map(|(_, combo)| combo)
}

fn build_ai_formed_units(
    player: &PlayerData,
    max_units: usize,
    cost_cap: f32,
    now: u64,
) -> Vec<StoredFormedUnit> {
    let locked = march_locked_card_slots(player, now);
    let mut free: Vec<usize> = (0..player.owned_cards.len())
        .filter(|i| !locked.contains(i))
        .collect();
    let mut used_slots = HashSet::new();
    let mut units = Vec::new();

    for unit_idx in 0..max_units {
        let candidates: Vec<usize> = free
            .iter()
            .copied()
            .filter(|i| !used_slots.contains(i))
            .collect();
        if candidates.is_empty() {
            break;
        }
        let Some(combo) = best_unit_combo(player, &candidates, cost_cap) else {
            break;
        };
        for &i in &combo {
            used_slots.insert(i);
            if let Some(pos) = free.iter().position(|&x| x == i) {
                free.remove(pos);
            }
        }
        units.push(StoredFormedUnit {
            id: format!("ai-unit-{unit_idx}"),
            name: format!("ユニット{}", unit_idx + 1),
            indices: slots_to_formed_indices(&combo),
        });
    }
    units
}

/// AI のユニット編成を施設上限まで維持（プレイヤーの `set_formed_units` と同じデータ）
pub(crate) fn ensure_ai_formed_units(player: &mut PlayerData, now: u64) {
    let bonuses = crate::facilities::calculate_facility_bonuses(&player.facilities);
    let max_units = (1 + bonuses.unit_capacity).max(1) as usize;
    let cost_cap = player.unit_cost_cap + bonuses.unit_cost_cap_bonus;
    if ai_formed_units_valid(player, max_units, now) {
        return;
    }
    player.formed_units = build_ai_formed_units(player, max_units, cost_cap, now);
}

pub(crate) fn initialize_ai_formed_units(player: &mut PlayerData) {
    ensure_ai_formed_units(player, default_now_ms());
}

fn formed_unit_available_for_dispatch(player: &PlayerData, unit: &StoredFormedUnit, now: u64) -> bool {
    if !is_kc_unit_ready_to_deploy(&unit.indices) {
        return false;
    }
    if march_busy_formed_unit_ids(player, now).contains(&unit.id) {
        return false;
    }
    let locked = march_locked_card_slots(player, now);
    formation_owned_slots_in_slot_order(&unit.indices)
        .iter()
        .all(|&slot| slot < player.owned_cards.len() && !locked.contains(&slot))
}

fn pick_ai_attack_from_formed_units(
    state: &GameState,
    ai_id: &str,
    target: &Territory,
    now: u64,
) -> Option<(String, AttackFormation)> {
    let player = state.players.get(ai_id)?;
    let (_, def_counts, def_names) = resolve_territory_defenders(target, &state.players);
    let mut ranked: Vec<(i64, String, AttackFormation)> = player
        .formed_units
        .iter()
        .filter(|u| formed_unit_available_for_dispatch(player, u, now))
        .filter_map(|u| {
            let slots = formation_owned_slots_in_slot_order(&u.indices);
            let formation = build_formation_from_indices(player, &slots)?;
            let score = score_formation(&slots, player, &def_names, &def_counts);
            Some((score, u.id.clone(), formation))
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, unit_id, formation) in ranked {
        let player = state.players.get(ai_id)?;
        if formation_viable_for_target(state, &formation, player, target) {
            return Some((unit_id, formation));
        }
    }
    None
}

fn unit_cost_cap_for(state: &GameState, player_id: &str) -> f32 {
    let facilities = state
        .players
        .get(player_id)
        .map(|p| p.facilities.as_slice())
        .unwrap_or(&[]);
    let bonuses = crate::facilities::calculate_facility_bonuses(facilities);
    state
        .players
        .get(player_id)
        .map(|p| p.unit_cost_cap)
        .unwrap_or(4.0)
        + bonuses.unit_cost_cap_bonus
}

fn card_effective_cost(
    player: &crate::model::PlayerData,
    card_index: usize,
) -> f32 {
    let card_id = player.owned_cards.get(card_index).copied().unwrap_or(0);
    let enhanced = player.card_enhanced.get(card_index).copied().unwrap_or(false);
    let base = get_card(card_id).map(|c| c.stats.cost).unwrap_or(1.5);
    if enhanced {
        base * 0.75
    } else {
        base
    }
}

fn formation_total_cost(
    player: &crate::model::PlayerData,
    indices: &[usize],
) -> f32 {
    indices
        .iter()
        .map(|&i| card_effective_cost(player, i))
        .sum()
}

/// 攻撃目標に対する最適編成（ユニットコスト上限内・1〜3体・同一種不可）
pub fn build_attack_formation(
    state: &GameState,
    ai_id: &str,
    target: &Territory,
) -> Option<AttackFormation> {
    let player = state.players.get(ai_id)?;
    let cost_cap = unit_cost_cap_for(state, ai_id);
    let now = crate::model::default_now_ms();
    let (_, def_counts, def_names) = resolve_territory_defenders(target, &state.players);

    let mut candidates: Vec<usize> = Vec::new();
    for i in 0..player.owned_cards.len() {
        if i < player.card_rest_until.len() && player.card_rest_until[i] > now {
            continue;
        }
        let stamina = player.card_stamina.get(i).copied().unwrap_or(0);
        if stamina < STAMINA_ATTACK {
            continue;
        }
        let count = player.card_monster_counts.get(i).copied().unwrap_or(0);
        if count == 0 {
            continue;
        }
        candidates.push(i);
    }
    if candidates.is_empty() {
        return None;
    }

    let max_bodies = candidates.len().min(3);
    let mut best: Option<(i64, Vec<usize>)> = None;

    for size in 1..=max_bodies {
        pick_combinations(&candidates, size, 0, &mut Vec::new(), &mut |combo: &[usize]| {
            let mut species = HashSet::new();
            for &idx in combo {
                if !species.insert(player.owned_cards[idx]) {
                    return;
                }
            }
            if formation_total_cost(player, combo) > cost_cap + 0.0001 {
                return;
            }
            let score = score_formation(combo, player, &def_names, &def_counts);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, combo.to_vec()));
            }
        });
    }

    let indices = best.map(|(_, idx)| idx)?;
    build_formation_from_indices(player, &indices)
}

fn pick_combinations<F>(candidates: &[usize], size: usize, start: usize, current: &mut Vec<usize>, f: &mut F)
where
    F: FnMut(&[usize]),
{
    if current.len() == size {
        f(current);
        return;
    }
    let need = size - current.len();
    for i in start..=candidates.len().saturating_sub(need) {
        current.push(candidates[i]);
        pick_combinations(candidates, size, i + 1, current, f);
        current.pop();
    }
}

fn build_formation_from_indices(
    player: &crate::model::PlayerData,
    indices: &[usize],
) -> Option<AttackFormation> {
    let mut card_indices = Vec::new();
    let mut monster_counts = Vec::new();
    let mut body_names = Vec::new();
    let mut speeds = Vec::new();
    let mut skills = Vec::new();
    let mut stats = Vec::new();

    for &idx in indices {
        let card_id = player.owned_cards.get(idx).copied()?;
        let card = get_card(card_id)?;
        let mc = player
            .card_monster_counts
            .get(idx)
            .copied()
            .unwrap_or(card.stats.monster_count)
            .max(1);
        card_indices.push(idx);
        monster_counts.push(mc);
        body_names.push(card.name.to_string());
        speeds.push(card.stats.speed);
        skills.push(get_card_skills(card_id));
        stats.push(card.stats.clone());
    }

    Some(AttackFormation {
        card_indices,
        monster_counts,
        body_names,
        speeds,
        skills,
        stats,
    })
}

fn score_formation(
    indices: &[usize],
    player: &crate::model::PlayerData,
    def_names: &[String],
    def_counts: &[u32],
) -> i64 {
    let mut score = 0i64;
    let mut species = HashSet::new();
    for &idx in indices {
        let card_id = player.owned_cards.get(idx).copied().unwrap_or(0);
        if !species.insert(card_id) {
            score -= 50;
        }
        let card = get_card(card_id);
        let atk = card.map(|c| c.stats.attack as i64).unwrap_or(10);
        let hp = card.map(|c| c.stats.defense as i64).unwrap_or(100);
        score += atk + hp / 10;
        let mc = player.card_monster_counts.get(idx).copied().unwrap_or(1) as i64;
        score += mc.min(500);
    }
    for (i, name) in def_names.iter().enumerate() {
        let def_power = def_counts.get(i).copied().unwrap_or(1) as i64 * 5;
        for &idx in indices {
            let card_id = player.owned_cards.get(idx).copied().unwrap_or(0);
            if counter_bonus(card_id, name) {
                score += 30 + def_power;
            }
        }
    }
    score
}

fn counter_bonus(attacker_card_id: u32, defender_name: &str) -> bool {
    let atk_name = get_card(attacker_card_id)
        .map(|c| c.name)
        .unwrap_or("");
    if defender_name.contains("スケルトン") && atk_name.contains("オーディン") {
        return true;
    }
    if defender_name.contains("ゴブリン") && attacker_card_id >= 12 {
        return true;
    }
    false
}

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

fn border_priority(state: &GameState, from_id: &str, target_id: &str) -> i32 {
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

fn score_attack_target(
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
        score -= 45;
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

fn target_on_cooldown(player: &crate::model::PlayerData, target_id: &str, now: u64) -> bool {
    player
        .ai_attack_cooldowns
        .iter()
        .any(|(id, until)| id == target_id && *until > now)
}

fn prune_ai_attack_cooldowns(player: &mut crate::model::PlayerData, now: u64) {
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

fn has_active_attack_march(state: &GameState, ai_id: &str) -> bool {
    state
        .players
        .get(ai_id)
        .map(|p| p.marches.iter().any(|m| m.kind == MarchKind::Attack))
        .unwrap_or(false)
}

fn formation_viable_for_target(
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

fn active_explore_march_count(state: &GameState, ai_id: &str) -> usize {
    state
        .players
        .get(ai_id)
        .map(|p| {
            p.marches
                .iter()
                .filter(|m| m.kind == MarchKind::Explore)
                .count()
        })
        .unwrap_or(0)
}

/// 探索の出発領地（隣接する自領。本拠が隣なら本拠から）
fn select_explore_from_territory(
    state: &GameState,
    ai_id: &str,
    to_id: &str,
) -> Option<String> {
    let home_id = state.players.get(ai_id)?.home_territory_id.clone();
    if territories_are_adjacent(&home_id, to_id) {
        return Some(home_id);
    }
    state
        .territories
        .iter()
        .filter(|t| t.owner_id.as_deref() == Some(ai_id) && t.id != to_id)
        .filter(|t| territories_are_adjacent(&t.id, to_id))
        .max_by_key(|t| t.troops)
        .map(|t| t.id.clone())
}

fn pick_exploration_territory(state: &GameState, ai_id: &str) -> Option<String> {
    let player = state.players.get(ai_id)?;
    let home_id = &player.home_territory_id;
    let in_flight: HashSet<String> = player
        .marches
        .iter()
        .filter(|m| m.kind == MarchKind::Explore)
        .map(|m| m.to_territory_id.clone())
        .collect();

    let mut candidates: Vec<&Territory> = state
        .territories
        .iter()
        .filter(|t| {
            t.owner_id.as_deref() == Some(ai_id)
                && !t.is_base
                && t.ruin.is_none()
                && t.id != *home_id
                && !in_flight.contains(&t.id)
                && select_explore_from_territory(state, ai_id, &t.id).is_some()
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }

    let low_resources = player.resources.food < 1200
        || player.resources.wood < 800
        || player.resources.stone < 800
        || player.resources.iron < 500;
    candidates.sort_by(|a, b| {
        let level_cmp = if low_resources {
            a.level.cmp(&b.level)
        } else {
            b.level.cmp(&a.level)
        };
        level_cmp.then_with(|| rand::random::<u8>().cmp(&rand::random()))
    });
    candidates.first().map(|t| t.id.clone())
}

fn run_exploration_dispatches(
    state: &mut GameState,
    ai_id: &str,
    dev_auto_win: bool,
    report: &mut AiTurnReport,
) {
    loop {
        let max_slots = state
            .players
            .get(ai_id)
            .map(|p| exploration_max_slots(p.exploration_level))
            .unwrap_or(1);
        let before_len = active_explore_march_count(state, ai_id);
        if before_len >= max_slots {
            break;
        }
        let Some(target_id) = pick_exploration_territory(state, ai_id) else {
            break;
        };
        let Some(from_id) = select_explore_from_territory(state, ai_id, &target_id) else {
            break;
        };
        let Some(card_indices) = pick_exploration_cards(state, ai_id) else {
            break;
        };

        let count = card_indices.len() as u32;
        let monsters_per_body: Vec<u32> = card_indices
            .iter()
            .map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.card_monster_counts.get(i))
                    .copied()
                    .unwrap_or(1)
            })
            .collect();
        let body_names: Vec<String> = card_indices
            .iter()
            .filter_map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.owned_cards.get(i))
                    .copied()
            })
            .filter_map(|cid| get_card(cid).map(|c| c.name.to_string()))
            .collect();
        let speed_per_body: Vec<u32> = card_indices
            .iter()
            .filter_map(|&i| {
                state
                    .players
                    .get(ai_id)
                    .and_then(|p| p.owned_cards.get(i))
                    .copied()
            })
            .map(|cid| get_card(cid).map(|c| c.stats.speed).unwrap_or(5))
            .collect();
        let next = apply_action(
            state,
            ai_id,
            &Action::StartMarch {
                kind: MarchKind::Explore,
                from_territory_id: from_id,
                to_territory_id: target_id.clone(),
                count,
                monsters_per_body: Some(monsters_per_body),
                body_names: Some(body_names),
                unit_name: Some(format!("{ai_id}探索")),
                speed_per_body: Some(speed_per_body),
                skills_per_body: None,
                stats_per_body: None,
                owned_card_indices: Some(card_indices),
                formed_unit_id: None,
            },
            dev_auto_win,
            ServerMode::Pve,
        );
        let after_len = active_explore_march_count(&next, ai_id);
        if after_len <= before_len {
            break;
        }
        if report.exploration_started.is_none() {
            report.exploration_started = Some(target_id);
        }
        *state = next;
    }
}

fn pick_exploration_cards(state: &GameState, ai_id: &str) -> Option<Vec<usize>> {
    let player = state.players.get(ai_id)?;
    let now = crate::model::default_now_ms();
    let locked = march_locked_card_slots(player, now);
    for (i, st) in player.card_stamina.iter().enumerate() {
        if locked.contains(&i) {
            continue;
        }
        if *st >= STAMINA_EXPLORE {
            if i < player.card_rest_until.len() && player.card_rest_until[i] > now {
                continue;
            }
            return Some(vec![i]);
        }
    }
    None
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
