use std::collections::HashSet;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::game_log::push_log;

use super::catalog::*;
use super::combat_character::*;
use super::types::*;

/// スキルの発動判定
pub fn should_trigger_skill(probability: u8) -> bool {
    if probability >= 100 {
        return true;
    }
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) < probability
}

fn get_triggered_skill(skill_id: &str, timing: SkillTiming, level: u8) -> Option<Skill> {
    let skill = get_skill(skill_id)?;
    let prob = adjusted_probability(skill.probability, level);
    if skill.timing == timing && should_trigger_skill(prob) {
        let multiplier = skill_level_multiplier(level);
        let mut scaled = skill;
        for effect in scaled.effects.iter_mut() {
            effect.value *= multiplier;
        }
        Some(scaled)
    } else {
        None
    }
}

fn get_triggered_active_skill(skill_id: &str, timing: SkillTiming, level: u8) -> Option<Skill> {
    let skill = get_skill(skill_id)?;
    if skill.category != SkillCategory::Active {
        return None;
    }
    let prob = adjusted_probability(skill.probability, level);
    if skill.timing == timing && should_trigger_skill(prob) {
        let multiplier = skill_level_multiplier(level);
        let mut scaled = skill;
        for effect in scaled.effects.iter_mut() {
            effect.value *= multiplier;
        }
        Some(scaled)
    } else {
        None
    }
}

pub(crate) fn side_label(is_ally: bool) -> &'static str {
    if is_ally {
        "[味方]"
    } else {
        "[敵]"
    }
}

fn log_skill_trigger(
    log: &mut Vec<String>,
    prefix: &str,
    is_ally: bool,
    character_name: &str,
    skill_name: &str,
    is_unique: bool,
) {
    let side = side_label(is_ally);
    if is_unique {
        push_log(
            log,
            format!(
                "{} {} {}の固有スキル「{}」が発動！",
                prefix, side, character_name, skill_name
            ),
        );
    } else {
        push_log(
            log,
            format!(
                "{} {} {}の「{}」が発動！",
                prefix, side, character_name, skill_name
            ),
        );
    }
}

/// KC準拠スタートアップフェーズ: 味方+敵を素早さ昇順で並べ、各自のスキルを自陣側に適用
pub fn apply_battle_start_skills(
    allies: &mut [CombatCharacter],
    enemies: &mut [CombatCharacter],
    log: &mut Vec<String>,
) {
    let mut startup_order: Vec<(usize, bool, f32, Option<String>, Option<String>, u8)> = Vec::new();
    for c in allies.iter() {
        if c.is_alive {
            startup_order.push((
                c.index,
                true,
                c.turn_order_speed(),
                c.skills.passive_id.clone(),
                c.skills.unique_id.clone(),
                skill_slot_level(&c.skills, 0),
            ));
        }
    }
    for c in enemies.iter() {
        if c.is_alive {
            startup_order.push((
                c.index,
                false,
                c.turn_order_speed(),
                c.skills.passive_id.clone(),
                c.skills.unique_id.clone(),
                skill_slot_level(&c.skills, 0),
            ));
        }
    }
    startup_order.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // 味方全体パッシブ（闘気の波動など）は同スキルIDの重複適用を1回に制限
    let mut applied_team_passives: (HashSet<String>, HashSet<String>) =
        (HashSet::new(), HashSet::new());

    for (char_idx, is_ally, _, passive_id, unique_id, level) in startup_order {
        if let Some(id) = passive_id {
            if let Some(skill) = get_triggered_skill(&id, SkillTiming::BattleStart, level) {
                let team_wide = skill
                    .effects
                    .iter()
                    .any(|e| e.target == SkillTarget::AllyUnit);
                if team_wide {
                    let applied = if is_ally {
                        &mut applied_team_passives.0
                    } else {
                        &mut applied_team_passives.1
                    };
                    if !applied.insert(id.clone()) {
                        continue;
                    }
                }
                let char_name = allies
                    .iter()
                    .chain(enemies.iter())
                    .find(|c| c.index == char_idx)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                log_skill_trigger(log, "◆", is_ally, &char_name, &skill.name, false);
                apply_skill_effects(&skill, char_idx, allies, enemies, is_ally, log);
            }
        }
        if let Some(id) = unique_id {
            if let Some(skill) = get_triggered_skill(&id, SkillTiming::BattleStart, level) {
                let team_wide = skill
                    .effects
                    .iter()
                    .any(|e| e.target == SkillTarget::AllyUnit);
                if team_wide {
                    let applied = if is_ally {
                        &mut applied_team_passives.0
                    } else {
                        &mut applied_team_passives.1
                    };
                    if !applied.insert(id.clone()) {
                        continue;
                    }
                }
                let char_name = allies
                    .iter()
                    .chain(enemies.iter())
                    .find(|c| c.index == char_idx)
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                log_skill_trigger(log, "◆◆", is_ally, &char_name, &skill.name, true);
                apply_skill_effects(&skill, char_idx, allies, enemies, is_ally, log);
            }
        }
    }
}

/// 攻撃時のスキル適用（アクティブは Skill1→Skill2→Skill3 の順に1つだけ発動、続けて固有ユニーク）
pub fn apply_attack_skills(
    attacker: &mut CombatCharacter,
    is_ally: bool,
    log: &mut Vec<String>,
) -> AttackModifiers {
    let mut modifiers = AttackModifiers::default();

    let slot_ids: [Option<&str>; 3] = [
        Some(attacker.skills.active_id.as_str()).filter(|s| !s.is_empty()),
        attacker.skills.skill2_id.as_deref(),
        attacker.skills.skill3_id.as_deref(),
    ];

    for (slot_i, id_opt) in slot_ids.into_iter().enumerate() {
        let Some(id) = id_opt else { continue };
        let level = skill_slot_level(&attacker.skills, slot_i);
        if let Some(skill) = get_triggered_active_skill(id, SkillTiming::OnAttack, level) {
            modifiers.skill_activated = true;
            log_skill_trigger(log, "★", is_ally, &attacker.name, &skill.name, false);
            for effect in &skill.effects {
                apply_attack_effect(effect, attacker, &mut modifiers, log);
            }
            return modifiers;
        }
    }

    if let Some(ref unique_id) = attacker.skills.unique_id {
        let level = skill_slot_level(&attacker.skills, 0);
        if let Some(skill) = get_triggered_skill(unique_id, SkillTiming::OnAttack, level) {
            if skill.category == SkillCategory::Unique {
                modifiers.skill_activated = true;
                log_skill_trigger(log, "★★", is_ally, &attacker.name, &skill.name, true);
                for effect in &skill.effects {
                    apply_attack_effect(effect, attacker, &mut modifiers, log);
                }
            }
        }
    }

    modifiers
}

/// 攻撃時の修正値
#[derive(Debug, Clone, Default)]
pub struct AttackModifiers {
    pub skill_activated: bool,
    pub damage_multiplier: f32,
    pub damage_add: f32,
    pub extra_attacks: u32,
    pub absorb_rate: f32,
    pub ignore_defense: bool,
    pub aoe_damage: f32,
    /// 敵全体への割合ダメージ（各対象の現在HPに対する比率）
    pub aoe_percent_damage: f32,
    pub true_damage: f32,
    pub percent_damage: f32,
    pub execute_threshold: f32,
    pub monster_steal: f32,
    pub status_effects: Vec<SkillEffect>,
    pub self_effects: Vec<SkillEffect>,
    pub ally_effects: Vec<SkillEffect>,
    pub heal_effects: Vec<SkillEffect>,
}

impl AttackModifiers {
    pub fn new() -> Self {
        Self {
            skill_activated: false,
            damage_multiplier: 1.0,
            damage_add: 0.0,
            extra_attacks: 0,
            absorb_rate: 0.0,
            ignore_defense: false,
            aoe_damage: 0.0,
            aoe_percent_damage: 0.0,
            true_damage: 0.0,
            percent_damage: 0.0,
            execute_threshold: 0.0,
            monster_steal: 0.0,
            status_effects: Vec::new(),
            self_effects: Vec::new(),
            ally_effects: Vec::new(),
            heal_effects: Vec::new(),
        }
    }
}

fn log_character_effects(log: &mut Vec<String>, name: &str, parts: Vec<String>) {
    if !parts.is_empty() {
        push_log(log, format!("  → {}: {}", name, parts.join("、")));
    }
}

fn apply_effects_to_character(
    character: &mut CombatCharacter,
    effects: &[&SkillEffect],
    log: &mut Vec<String>,
) {
    let mut parts = Vec::new();
    let mut overflow = Vec::new();
    for effect in effects {
        if let Some(detail) = apply_effect_to_character_core(effect, character, &mut overflow) {
            parts.push(detail);
        }
    }
    log_character_effects(log, &character.name, parts);
    for line in overflow {
        push_log(log, line);
    }
}

/// スタートアップ等でスキル効果を適用（味方・敵スライスを分けて Enemy* / Ally* を解決）
pub(crate) fn apply_skill_effects(
    skill: &Skill,
    caster_idx: usize,
    allies: &mut [CombatCharacter],
    enemies: &mut [CombatCharacter],
    caster_on_allies: bool,
    log: &mut Vec<String>,
) {
    let (friend, foe): (&mut [CombatCharacter], &mut [CombatCharacter]) = if caster_on_allies {
        (allies, enemies)
    } else {
        (enemies, allies)
    };

    let mut targets: Vec<SkillTarget> = Vec::new();
    for effect in &skill.effects {
        if !targets.iter().any(|t| *t == effect.target) {
            targets.push(effect.target.clone());
        }
    }

    for target in targets {
        let effects: Vec<&SkillEffect> = skill
            .effects
            .iter()
            .filter(|e| e.target == target)
            .collect();

        match target {
            SkillTarget::SelfOnly => {
                if let Some(c) = friend.iter_mut().find(|c| c.index == caster_idx) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
            SkillTarget::AllyUnit => {
                for c in friend.iter_mut().filter(|c| c.is_alive) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
            SkillTarget::AllySingle => {
                let alive: Vec<usize> = friend
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_alive)
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&idx) = alive.choose(&mut rand::thread_rng()) {
                    apply_effects_to_character(&mut friend[idx], &effects, log);
                }
            }
            SkillTarget::AllyLowestHp => {
                if let Some(c) = friend.iter_mut().filter(|c| c.is_alive).min_by(|a, b| {
                    a.current_monster_count
                        .partial_cmp(&b.current_monster_count)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
            SkillTarget::EnemySingle | SkillTarget::EnemyRandom => {
                let alive: Vec<usize> = foe
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.is_alive)
                    .map(|(i, _)| i)
                    .collect();
                if let Some(&idx) = alive.choose(&mut rand::thread_rng()) {
                    apply_effects_to_character(&mut foe[idx], &effects, log);
                }
            }
            SkillTarget::EnemyHighestHp => {
                if let Some(c) = foe.iter_mut().filter(|c| c.is_alive).max_by(|a, b| {
                    a.current_monster_count
                        .partial_cmp(&b.current_monster_count)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
            SkillTarget::EnemyAll => {
                for c in foe.iter_mut().filter(|c| c.is_alive) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
            SkillTarget::BothAll => {
                for c in friend.iter_mut().chain(foe.iter_mut()).filter(|c| c.is_alive) {
                    apply_effects_to_character(c, &effects, log);
                }
            }
        }
    }
}

pub fn apply_effect_to_character(
    effect: &SkillEffect,
    character: &mut CombatCharacter,
    log: &mut Vec<String>,
) {
    let mut overflow = Vec::new();
    if let Some(detail) = apply_effect_to_character_core(effect, character, &mut overflow) {
        log_character_effects(log, &character.name, vec![detail]);
    }
    for line in overflow {
        push_log(log, line);
    }
}

fn apply_effect_to_character_core(
    effect: &SkillEffect,
    character: &mut CombatCharacter,
    overflow_log: &mut Vec<String>,
) -> Option<String> {
    let turns = effect.duration.as_ref().and_then(|d| d.turns).unwrap_or(99);

    match effect.effect_type {
        // ステータス変更系
        SkillEffectType::MonsterCountMultiply => {
            let before = character.current_monster_count;
            character.current_monster_count *= effect.value;
            Some(format!(
                "魔獣数 {:.0} → {:.0}",
                before, character.current_monster_count
            ))
        }
        SkillEffectType::MonsterCountAdd => {
            let before = character.current_monster_count;
            character.current_monster_count += effect.value;
            Some(format!(
                "魔獣数 {:.0} → {:.0}",
                before, character.current_monster_count
            ))
        }
        SkillEffectType::MonsterCountSet => {
            let before = character.current_monster_count;
            character.current_monster_count = effect.value;
            Some(format!(
                "魔獣数 {:.0} → {:.0}",
                before, character.current_monster_count
            ))
        }
        SkillEffectType::SpeedMultiply => {
            let before = character.current_speed;
            character.current_speed *= effect.value;
            Some(format!(
                "SPEED {:.0} → {:.0}",
                before, character.current_speed
            ))
        }
        SkillEffectType::SpeedAdd => {
            let before = character.current_speed;
            character.current_speed += effect.value;
            Some(format!(
                "SPEED {:.0} → {:.0}",
                before, character.current_speed
            ))
        }
        SkillEffectType::DamageMultiply => {
            character.damage_multiplier *= effect.value;
            Some(format!("ダメージ倍率 x{:.1}", character.damage_multiplier))
        }
        SkillEffectType::DamageReduce => {
            character.damage_reduction += effect.value;
            if effect.value > 0.0 {
                Some(format!(
                    "被ダメージ軽減 {:.0}%",
                    character.damage_reduction * 100.0
                ))
            } else {
                Some(format!("被ダメージ増加 +{:.0}%", -effect.value * 100.0))
            }
        }
        SkillEffectType::DamageReflect => {
            character.status_effects.push(StatusEffect::DamageReflect { rate: effect.value, turns });
            Some(format!("ダメージ反射{:.0}%付与", effect.value * 100.0))
        }
        // 防御系
        SkillEffectType::Shield => {
            character.status_effects.push(StatusEffect::Shield { amount: effect.value });
            Some(format!("シールド{:.0}付与", effect.value))
        }
        SkillEffectType::Invincible => {
            character.status_effects.push(StatusEffect::Invincible { count: effect.value as u8 });
            Some(format!("無敵（{:.0}回）付与", effect.value))
        }
        SkillEffectType::Evasion => {
            character.status_effects.push(StatusEffect::Evasion { rate: effect.value, turns });
            Some(format!("回避率+{:.0}%付与", effect.value * 100.0))
        }
        SkillEffectType::Counter => {
            character.status_effects.push(StatusEffect::Counter { rate: effect.value, turns });
            Some(format!("反撃態勢（{:.0}%）付与", effect.value * 100.0))
        }
        // 状態異常系
        SkillEffectType::Poison => {
            character.status_effects.push(StatusEffect::Poison { damage: effect.value, turns });
            Some(format!("毒付与（毎ターン {:.0} ダメージ）", effect.value))
        }
        SkillEffectType::Burn => {
            character.status_effects.push(StatusEffect::Burn { damage: effect.value, turns });
            Some(format!("炎上付与（毎ターン {:.0} ダメージ）", effect.value))
        }
        SkillEffectType::Freeze => {
            character.status_effects.push(StatusEffect::Freeze { turns });
            Some(format!("凍結（{}ターン行動不能）", turns))
        }
        SkillEffectType::Stun => {
            character.status_effects.push(StatusEffect::Stun { turns });
            Some(format!("気絶（{}ターン行動不能）", turns))
        }
        SkillEffectType::Silence => {
            character.status_effects.push(StatusEffect::Silence { turns });
            Some(format!("沈黙（{}ターンスキル使用不可）", turns))
        }
        SkillEffectType::Blind => {
            character.status_effects.push(StatusEffect::Blind { miss_rate: effect.value, turns });
            Some(format!("暗闘付与（命中率{:.0}%低下）", effect.value * 100.0))
        }
        SkillEffectType::Confuse => {
            let ch = effect.value.clamp(0.0, 1.0);
            character
                .status_effects
                .push(StatusEffect::Confused { hit_own_team_chance: ch, turns });
            Some(format!("混乱付与（自陣誤射{:.0}%）", ch * 100.0))
        }
        SkillEffectType::Charm => {
            character.status_effects.push(StatusEffect::Charmed { turns });
            Some(format!("魅了（{}ターン）", turns))
        }
        SkillEffectType::Weaken => {
            character.status_effects.push(StatusEffect::Weaken { reduction: effect.value, turns });
            Some(format!("弱体化付与（ダメージ{:.0}%低下）", effect.value * 100.0))
        }
        SkillEffectType::Vulnerable => {
            character.status_effects.push(StatusEffect::Vulnerable { increase: effect.value, turns });
            Some(format!("脆弱付与（被ダメージ{:.0}%増加）", effect.value * 100.0))
        }
        SkillEffectType::Mark => {
            character.status_effects.push(StatusEffect::Mark { bonus_damage: effect.value, turns });
            Some(format!("マーク付与（被ダメージ+{:.0}）", effect.value))
        }
        // バフ系
        SkillEffectType::AttackBuff => {
            character.status_effects.push(StatusEffect::AttackBuff { bonus: effect.value, turns });
            Some(format!("攻撃バフ+{:.0}%付与", effect.value * 100.0))
        }
        SkillEffectType::DefenseBuff => {
            character.status_effects.push(StatusEffect::DefenseBuff { bonus: effect.value, turns });
            Some(format!("防御バフ+{:.0}%付与", effect.value * 100.0))
        }
        SkillEffectType::SpeedBuff => {
            character.status_effects.push(StatusEffect::SpeedBuff { bonus: effect.value, turns });
            Some(format!("速度バフ+{:.0}%付与", effect.value * 100.0))
        }
        // 特殊系
        SkillEffectType::Taunt => {
            character.status_effects.push(StatusEffect::Taunt { turns });
            Some("挑発付与（敵の攻撃を引きつける）".to_string())
        }
        SkillEffectType::Stealth => {
            character.status_effects.push(StatusEffect::Stealth { turns });
            Some("隠密状態".to_string())
        }
        SkillEffectType::ExtraAttack => {
            character.extra_attacks += effect.value as u32;
            Some(format!("追加攻撃{:.0}回付与", effect.value))
        }
        SkillEffectType::Heal => {
            let before = character.current_monster_count;
            character.current_monster_count = (character.current_monster_count + effect.value)
                .min(character.base_monster_count as f32);
            Some(format!(
                "{:.0} 回復（{:.0} → {:.0}）",
                effect.value, before, character.current_monster_count
            ))
        }
        SkillEffectType::HealPercent => {
            let before = character.current_monster_count;
            let heal_amount = character.base_monster_count as f32 * effect.value;
            character.current_monster_count = (character.current_monster_count + heal_amount)
                .min(character.base_monster_count as f32);
            Some(format!(
                "{:.0} 回復（{:.0} → {:.0}）（{:.0}%）",
                heal_amount, before, character.current_monster_count, effect.value * 100.0
            ))
        }
        SkillEffectType::PercentDamage => {
            let damage = character.current_monster_count * effect.value;
            character.current_monster_count -= damage;
            let detail = format!("{:.0} ダメージ（{:.0}%）", damage, effect.value * 100.0);
            if character.current_monster_count <= 0.0 {
                character.is_alive = false;
                overflow_log.push(format!("  → {}が倒れた！", character.name));
            }
            Some(detail)
        }
        SkillEffectType::Cleanse => {
            let count = effect.value as usize;
            let mut removed = 0;
            character.status_effects.retain(|e| {
                if removed >= count {
                    return true;
                }
                match e {
                    StatusEffect::Poison { .. }
                    | StatusEffect::Burn { .. }
                    | StatusEffect::Freeze { .. }
                    | StatusEffect::Stun { .. }
                    | StatusEffect::Silence { .. }
                    | StatusEffect::Blind { .. }
                    | StatusEffect::Confused { .. }
                    | StatusEffect::Charmed { .. }
                    | StatusEffect::Weaken { .. }
                    | StatusEffect::Vulnerable { .. }
                    | StatusEffect::Mark { .. } => {
                        removed += 1;
                        false
                    }
                    _ => true,
                }
            });
            if removed > 0 {
                Some(format!("デバフを{}個解除", removed))
            } else {
                None
            }
        }
        SkillEffectType::Dispel => {
            let count = effect.value as usize;
            let mut removed = 0;
            character.status_effects.retain(|e| {
                if removed >= count {
                    return true;
                }
                match e {
                    StatusEffect::AttackBuff { .. }
                    | StatusEffect::DefenseBuff { .. }
                    | StatusEffect::SpeedBuff { .. }
                    | StatusEffect::Shield { .. }
                    | StatusEffect::Invincible { .. }
                    | StatusEffect::Evasion { .. }
                    | StatusEffect::Counter { .. }
                    | StatusEffect::DamageReflect { .. }
                    | StatusEffect::Stealth { .. } => {
                        removed += 1;
                        false
                    }
                    _ => true,
                }
            });
            if removed > 0 {
                Some(format!("バフを{}個解除", removed))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn apply_attack_effect(
    effect: &SkillEffect,
    attacker: &CombatCharacter,
    modifiers: &mut AttackModifiers,
    log: &mut Vec<String>,
) {
    match effect.effect_type {
        SkillEffectType::DamageMultiply => {
            modifiers.damage_multiplier *= effect.value;
            push_log(log, format!("  → ダメージ x{:.1}倍", effect.value));
        }
        SkillEffectType::DamageAdd => {
            if effect.target == SkillTarget::EnemySingle && effect.value < 1.0 {
                let bonus = attacker.current_speed * effect.value;
                modifiers.damage_add += bonus;
                push_log(log, format!("  → +{:.0} ダメージ（SPEED補正）", bonus));
            } else {
                modifiers.damage_add += effect.value;
                push_log(log, format!("  → +{:.0} ダメージ", effect.value));
            }
        }
        SkillEffectType::ExtraAttack => {
            modifiers.extra_attacks += effect.value as u32;
            push_log(log, "  → 追加攻撃発生！".to_string());
        }
        SkillEffectType::Absorb => {
            modifiers.absorb_rate += effect.value;
            push_log(log, format!("  → 与ダメージの{:.0}%を吸収", effect.value * 100.0));
        }
        SkillEffectType::TrueDamage => {
            modifiers.ignore_defense = true;
            if effect.value > 0.0 {
                if effect.target == SkillTarget::EnemyAll {
                    modifiers.aoe_damage += effect.value;
                    push_log(log, format!("  → +{:.0} 全体固定ダメージ", effect.value));
                } else {
                    modifiers.true_damage += effect.value;
                    push_log(log, format!("  → +{:.0} 固定ダメージ", effect.value));
                }
            } else {
                push_log(log, "  → 防御無視".to_string());
            }
        }
        SkillEffectType::PercentDamage => {
            modifiers.percent_damage += effect.value;
            push_log(log, format!("  → +{:.0}% 割合ダメージ", effect.value * 100.0));
        }
        SkillEffectType::Execute => {
            modifiers.execute_threshold = modifiers.execute_threshold.max(effect.value);
            push_log(log, format!("  → HP{:.0}%以下の敵を即死", effect.value * 100.0));
        }
        SkillEffectType::MonsterCountSteal => {
            modifiers.monster_steal += effect.value;
            push_log(log, format!("  → {:.0} 魔獣数奪取", effect.value));
        }
        // 状態異常付与系は別途処理
        SkillEffectType::Poison | SkillEffectType::Burn | SkillEffectType::Freeze |
        SkillEffectType::Stun | SkillEffectType::Silence | SkillEffectType::Vulnerable |
        SkillEffectType::Mark | SkillEffectType::Weaken | SkillEffectType::Confuse |
        SkillEffectType::Charm => {
            modifiers.status_effects.push(effect.clone());
        }
        // バフ系
        SkillEffectType::Shield => {
            modifiers.self_effects.push(effect.clone());
            push_log(log, format!("  → シールド{:.0}付与", effect.value));
        }
        SkillEffectType::AttackBuff => {
            modifiers.ally_effects.push(effect.clone());
            push_log(log, format!("  → 味方に攻撃バフ+{:.0}%", effect.value * 100.0));
        }
        SkillEffectType::Heal => {
            modifiers.heal_effects.push(effect.clone());
            push_log(log, format!("  → {:.0} 回復", effect.value));
        }
        SkillEffectType::Cleanse | SkillEffectType::Dispel => {
            modifiers.self_effects.push(effect.clone());
        }
        SkillEffectType::Taunt => {
            modifiers.self_effects.push(effect.clone());
            push_log(log, "  → 挑発付与".to_string());
        }
        _ => {}
    }

    if effect.target == SkillTarget::EnemyAll {
        match effect.effect_type {
            SkillEffectType::PercentDamage => {
                modifiers.aoe_percent_damage += effect.value;
            }
            SkillEffectType::DamageAdd => {
                modifiers.aoe_damage += effect.value;
            }
            _ => {}
        }
    }
}

/// 死亡時スキル（復活など）のチェック
pub fn check_death_skills(
    character: &mut CombatCharacter,
    log: &mut Vec<String>,
) -> bool {
    if let Some(ref unique_id) = character.skills.unique_id {
        let level = skill_slot_level(&character.skills, 0);
        if let Some(skill) = get_triggered_skill(unique_id, SkillTiming::OnDeath, level) {
            for effect in &skill.effects {
                if effect.effect_type == SkillEffectType::Revive {
                    character.current_monster_count = character.base_monster_count as f32 * effect.value;
                    character.is_alive = true;
                    push_log(
                        log,
                        format!(
                            "{}の「{}」が発動！{} 魔獣数で復活！",
                            character.name,
                            skill.name,
                            character.effective_monster_count()
                        ),
                    );
                    return true;
                }
            }
        }
    }
    false
}
