use serde::{Deserialize, Serialize};

/// スキルの発動タイミング
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillTiming {
    BattleStart,
    OnAttack,
    OnDefend,
    OnDamage,
    OnKill,
    OnDeath,
    TurnStart,
    TurnEnd,
    HpLow,
    FirstAttack,
    Continuous,
}

/// スキルの効果対象
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillTarget {
    #[serde(rename = "self")]
    SelfOnly,
    AllyUnit,
    AllySingle,
    AllyLowestHp,
    EnemySingle,
    EnemyAll,
    EnemyRandom,
    EnemyHighestHp,
    BothAll,
}

/// スキルの効果タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillEffectType {
    // ステータス変更系
    #[serde(alias = "energy_multiply")]
    MonsterCountMultiply,
    #[serde(alias = "energy_add")]
    MonsterCountAdd,
    #[serde(alias = "energy_set")]
    MonsterCountSet,
    SpeedMultiply,
    SpeedAdd,
    DamageMultiply,
    DamageAdd,
    DamageReduce,
    DamageReflect,
    // 回復・蘇生系
    Heal,
    HealPercent,
    Revive,
    Absorb,
    // 攻撃系
    ExtraAttack,
    TrueDamage,
    PercentDamage,
    Execute,
    // 防御系
    Shield,
    Invincible,
    Evasion,
    Counter,
    // 状態異常系
    Poison,
    Burn,
    Freeze,
    Stun,
    Silence,
    Blind,
    /// 混乱: value=味方（自陣）を殴る確率 0.0-1.0
    Confuse,
    /// 魅了: 行動が常に相手有利（自陣を攻撃）
    Charm,
    Weaken,
    Vulnerable,
    // バフ・デバフ系
    AttackBuff,
    DefenseBuff,
    SpeedBuff,
    CriticalBuff,
    AttackDebuff,
    DefenseDebuff,
    Cleanse,
    Dispel,
    // 特殊系
    Taunt,
    Stealth,
    Mark,
    CopyBuff,
    TransferDebuff,
    CooldownReduce,
    #[serde(alias = "energy_steal")]
    MonsterCountSteal,
    Summon,
}

/// 効果の持続情報
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EffectDuration {
    #[serde(default)]
    pub turns: Option<u8>,
    #[serde(default)]
    pub stacks: Option<u8>,
    #[serde(default)]
    pub max_stacks: Option<u8>,
}

/// スキル効果の定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillEffect {
    #[serde(rename = "type")]
    pub effect_type: SkillEffectType,
    pub target: SkillTarget,
    pub value: f32,
    #[serde(default)]
    pub duration: Option<EffectDuration>,
}

/// スキルの種別
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Passive,
    Active,
    Unique,
}

/// スキル定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: SkillCategory,
    pub timing: SkillTiming,
    pub effects: Vec<SkillEffect>,
    #[serde(default = "default_probability")]
    pub probability: u8,
}

fn default_probability() -> u8 {
    100
}

fn default_skill_level() -> u8 {
    1
}

fn default_slot_levels() -> [u8; 3] {
    [1, 1, 1]
}

/// クライアントから送信されるスキルデータ（Skill1=固有アクティブ、Skill2/3=合成追加）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub passive_id: Option<String>,
    pub active_id: String,
    pub unique_id: Option<String>,
    /// 後方互換: スロット1のLvとしても扱う
    #[serde(default = "default_skill_level")]
    pub skill_level: u8,
    #[serde(default)]
    pub skill2_id: Option<String>,
    #[serde(default)]
    pub skill3_id: Option<String>,
    /// 各スロットのスキルLv（1-10）
    #[serde(default = "default_slot_levels")]
    pub slot_levels: [u8; 3],
}

impl Default for SkillData {
    fn default() -> Self {
        Self {
            passive_id: None,
            active_id: String::new(),
            unique_id: None,
            skill_level: 1,
            skill2_id: None,
            skill3_id: None,
            slot_levels: [1, 1, 1],
        }
    }
}

/// 所持スロットの合成スキルLvを SkillData に反映（未登録時は Lv1 のまま）
pub fn apply_owned_card_skill_levels(
    skill: &mut SkillData,
    card_idx: usize,
    card_skill_levels: &std::collections::HashMap<usize, [u8; 3]>,
) {
    let Some(levels) = card_skill_levels.get(&card_idx) else {
        return;
    };
    skill.slot_levels = levels.map(|lv| if lv == 0 { 1 } else { lv.clamp(1, 10) });
    skill.skill_level = skill.slot_levels[0];
}

/// スロット index (0..3) の実効スキルLv
pub fn skill_slot_level(skills: &SkillData, slot: usize) -> u8 {
    let from_arr = skills
        .slot_levels
        .get(slot)
        .copied()
        .unwrap_or(1)
        .clamp(1, 10);
    if slot == 0 {
        if from_arr == 1 && skills.skill_level > 1 {
            skills.skill_level.clamp(1, 10)
        } else {
            from_arr
        }
    } else {
        from_arr
    }
}

/// スキルレベルによる効果倍率（Lv1〜7は着実に成長、Lv8〜10で大幅強化）
pub fn skill_level_multiplier(level: u8) -> f32 {
    match level.clamp(1, 10) {
        1 => 1.00,
        2 => 1.12,
        3 => 1.24,
        4 => 1.36,
        5 => 1.48,
        6 => 1.60,
        7 => 1.75,
        8 => 2.20,
        9 => 2.80,
        10 => 3.50,
        _ => 1.00,
    }
}

/// スキルレベルによる発動確率補正（効果倍率と同じカーブで上限100%へ近づく）
pub fn adjusted_probability(base_prob: u8, level: u8) -> u8 {
    let base = base_prob.min(100);
    if base >= 100 {
        return 100;
    }
    let gap = (100 - base) as f32;
    let mult = skill_level_multiplier(level);
    let fill = ((mult - 1.0) / 2.5).clamp(0.0, 1.0);
    (base as f32 + gap * fill).round().clamp(0.0, 100.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_skill_levels_jump_at_eight_nine_ten() {
        assert!((skill_level_multiplier(7) - 1.75).abs() < f32::EPSILON);
        assert!((skill_level_multiplier(8) - 2.20).abs() < f32::EPSILON);
        assert!((skill_level_multiplier(9) - 2.80).abs() < f32::EPSILON);
        assert!((skill_level_multiplier(10) - 3.50).abs() < f32::EPSILON);
        assert!(skill_level_multiplier(7) > skill_level_multiplier(4));
        assert!(skill_level_multiplier(10) > skill_level_multiplier(7) * 1.5);
    }

    #[test]
    fn probability_reaches_cap_at_level_ten_for_partial_skills() {
        assert_eq!(adjusted_probability(30, 10), 100);
        assert!(adjusted_probability(30, 8) > adjusted_probability(30, 7));
    }
}
