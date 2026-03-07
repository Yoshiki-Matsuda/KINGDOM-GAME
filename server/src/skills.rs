//! スキルシステム — パッシブ・アクティブ・ユニークスキルの定義と効果処理
//!
//! - パッシブスキル: 戦闘開始時に発動。味方ユニット全体に効果
//! - アクティブスキル: 攻撃時に発動。全キャラが持つ
//! - ユニークスキル: 特別キャラ専用。発動タイミングはスキル自体に定義

use rand::Rng;
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
    EnergyMultiply,
    EnergyAdd,
    EnergySet,
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
    EnergySteal,
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

/// クライアントから送信されるスキルデータ
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillData {
    pub passive_id: Option<String>,
    pub active_id: String,
    pub unique_id: Option<String>,
}

/// 状態異常・バフの種類
#[derive(Debug, Clone, PartialEq)]
pub enum StatusEffect {
    // 状態異常（デバフ）
    Poison { damage: f32, turns: u8 },
    Burn { damage: f32, turns: u8 },
    Freeze { turns: u8 },
    Stun { turns: u8 },
    Silence { turns: u8 },
    Blind { miss_rate: f32, turns: u8 },
    Weaken { reduction: f32, turns: u8 },
    Vulnerable { increase: f32, turns: u8 },
    Mark { bonus_damage: f32, turns: u8 },
    // バフ
    AttackBuff { bonus: f32, turns: u8 },
    DefenseBuff { bonus: f32, turns: u8 },
    SpeedBuff { bonus: f32, turns: u8 },
    Shield { amount: f32 },
    Invincible { count: u8 },
    Evasion { rate: f32, turns: u8 },
    Counter { rate: f32, turns: u8 },
    DamageReflect { rate: f32, turns: u8 },
    Taunt { turns: u8 },
    Stealth { turns: u8 },
}

/// 戦闘中のキャラクター状態（スキル適用後の修正値を含む）
#[derive(Debug, Clone)]
pub struct CombatCharacter {
    pub index: usize,
    pub name: String,
    pub base_energy: u32,
    pub current_energy: f32,
    pub base_speed: u32,
    pub current_speed: f32,
    /// 攻撃力（物理ダメージ）
    pub attack: u32,
    /// 魔力（魔法ダメージ）
    pub magic: u32,
    /// 防御力（物理防御）
    pub defense: u32,
    /// 魔法防御力
    pub magic_defense: u32,
    pub skills: SkillData,
    pub damage_multiplier: f32,
    pub damage_reduction: f32,
    pub extra_attacks: u32,
    pub is_alive: bool,
    pub is_first_attack: bool,
    pub status_effects: Vec<StatusEffect>,
    pub has_revived: bool,
}

impl CombatCharacter {
    pub fn new(index: usize, name: String, energy: u32, speed: u32, skills: SkillData) -> Self {
        Self {
            index,
            name,
            base_energy: energy,
            current_energy: energy as f32,
            base_speed: speed,
            current_speed: speed as f32,
            attack: 5,
            magic: 5,
            defense: 3,
            magic_defense: 3,
            skills,
            damage_multiplier: 1.0,
            damage_reduction: 0.0,
            extra_attacks: 0,
            is_alive: true,
            is_first_attack: true,
            status_effects: Vec::new(),
            has_revived: false,
        }
    }

    /// 全ステータスを指定してキャラクター作成
    pub fn with_stats(
        index: usize,
        name: String,
        energy: u32,
        speed: u32,
        attack: u32,
        magic: u32,
        defense: u32,
        magic_defense: u32,
        skills: SkillData,
    ) -> Self {
        Self {
            index,
            name,
            base_energy: energy,
            current_energy: energy as f32,
            base_speed: speed,
            current_speed: speed as f32,
            attack,
            magic,
            defense,
            magic_defense,
            skills,
            damage_multiplier: 1.0,
            damage_reduction: 0.0,
            extra_attacks: 0,
            is_alive: true,
            is_first_attack: true,
            status_effects: Vec::new(),
            has_revived: false,
        }
    }

    /// 物理ダメージを計算
    pub fn physical_damage(&self) -> f32 {
        self.attack as f32 * self.damage_multiplier
    }

    /// 魔法ダメージを計算
    pub fn magic_damage(&self) -> f32 {
        self.magic as f32 * self.damage_multiplier
    }

    /// 合計攻撃力（物理+魔法、簡易計算）
    pub fn total_attack_power(&self) -> f32 {
        (self.attack as f32 + self.magic as f32) * self.damage_multiplier
    }

    pub fn effective_energy(&self) -> u32 {
        (self.current_energy.max(0.0).round()) as u32
    }

    /// シールド量を取得
    pub fn get_shield(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Shield { amount } = e { Some(*amount) } else { None }
        }).sum()
    }

    /// 無敵回数を取得
    pub fn get_invincible_count(&self) -> u8 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Invincible { count } = e { Some(*count) } else { None }
        }).sum()
    }

    /// 回避率を取得
    pub fn get_evasion_rate(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Evasion { rate, .. } = e { Some(*rate) } else { None }
        }).sum()
    }

    /// 行動不能かどうか
    pub fn is_disabled(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Freeze { .. } | StatusEffect::Stun { .. }))
    }

    /// 沈黙状態かどうか
    pub fn is_silenced(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Silence { .. }))
    }

    /// 隠密状態かどうか
    pub fn is_stealthed(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Stealth { .. }))
    }

    /// 挑発状態かどうか
    pub fn is_taunting(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Taunt { .. }))
    }

    /// 被ダメージ増加率を取得
    pub fn get_vulnerability(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Vulnerable { increase, .. } = e { Some(*increase) } else { None }
        }).sum()
    }

    /// マークによる追加ダメージを取得
    pub fn get_mark_damage(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Mark { bonus_damage, .. } = e { Some(*bonus_damage) } else { None }
        }).sum()
    }

    /// 反撃率を取得
    pub fn get_counter_rate(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::Counter { rate, .. } = e { Some(*rate) } else { None }
        }).sum()
    }

    /// ダメージ反射率を取得
    pub fn get_reflect_rate(&self) -> f32 {
        self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::DamageReflect { rate, .. } = e { Some(*rate) } else { None }
        }).sum()
    }

    /// ターン経過処理（毒・炎上ダメージ、持続ターン減少）
    pub fn process_turn_effects(&mut self, log: &mut Vec<String>) {
        let mut damage_taken = 0.0;

        for effect in &self.status_effects {
            match effect {
                StatusEffect::Poison { damage, .. } => {
                    damage_taken += damage;
                    log.push(format!("  {}が毒で{:.0}ダメージ！", self.name, damage));
                }
                StatusEffect::Burn { damage, .. } => {
                    damage_taken += damage;
                    log.push(format!("  {}が炎上で{:.0}ダメージ！", self.name, damage));
                }
                _ => {}
            }
        }

        if damage_taken > 0.0 {
            self.current_energy -= damage_taken;
            if self.current_energy <= 0.0 {
                self.is_alive = false;
                log.push(format!("  {}が状態異常で倒れた！", self.name));
            }
        }

        // ターン経過で効果を減少
        self.status_effects.retain_mut(|effect| {
            match effect {
                StatusEffect::Poison { turns, .. } |
                StatusEffect::Burn { turns, .. } |
                StatusEffect::Freeze { turns } |
                StatusEffect::Stun { turns } |
                StatusEffect::Silence { turns } |
                StatusEffect::Blind { turns, .. } |
                StatusEffect::Weaken { turns, .. } |
                StatusEffect::Vulnerable { turns, .. } |
                StatusEffect::Mark { turns, .. } |
                StatusEffect::AttackBuff { turns, .. } |
                StatusEffect::DefenseBuff { turns, .. } |
                StatusEffect::SpeedBuff { turns, .. } |
                StatusEffect::Evasion { turns, .. } |
                StatusEffect::Counter { turns, .. } |
                StatusEffect::DamageReflect { turns, .. } |
                StatusEffect::Taunt { turns } |
                StatusEffect::Stealth { turns } => {
                    *turns = turns.saturating_sub(1);
                    *turns > 0
                }
                StatusEffect::Shield { amount } => *amount > 0.0,
                StatusEffect::Invincible { count } => *count > 0,
            }
        });
    }

    /// シールドでダメージを吸収
    pub fn absorb_damage_with_shield(&mut self, damage: f32) -> f32 {
        let mut remaining = damage;
        for effect in &mut self.status_effects {
            if let StatusEffect::Shield { amount } = effect {
                if *amount >= remaining {
                    *amount -= remaining;
                    return 0.0;
                } else {
                    remaining -= *amount;
                    *amount = 0.0;
                }
            }
        }
        remaining
    }

    /// 無敵で攻撃を無効化
    pub fn consume_invincible(&mut self) -> bool {
        for effect in &mut self.status_effects {
            if let StatusEffect::Invincible { count } = effect {
                if *count > 0 {
                    *count -= 1;
                    return true;
                }
            }
        }
        false
    }
}

/// スキルマスターデータを取得
pub fn get_skill(id: &str) -> Option<Skill> {
    match id {
        // パッシブスキル
        // === パッシブスキル ===
        "power_aura" => Some(Skill {
            id: "power_aura".to_string(),
            name: "闘気の波動".to_string(),
            description: "戦闘開始時、味方全員のエナジー1.2倍".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergyMultiply,
                target: SkillTarget::AllyUnit,
                value: 1.2,
                duration: None,
            }],
            probability: 100,
        }),
        "wind_blessing" => Some(Skill {
            id: "wind_blessing".to_string(),
            name: "疾風の祝福".to_string(),
            description: "戦闘開始時、味方全員のSPEED+2".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::SpeedAdd,
                target: SkillTarget::AllyUnit,
                value: 2.0,
                duration: None,
            }],
            probability: 100,
        }),
        "life_blessing" => Some(Skill {
            id: "life_blessing".to_string(),
            name: "生命の恵み".to_string(),
            description: "戦闘開始時、味方全員のエナジー+3".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergyAdd,
                target: SkillTarget::AllyUnit,
                value: 3.0,
                duration: None,
            }],
            probability: 100,
        }),
        "rage_aura" => Some(Skill {
            id: "rage_aura".to_string(),
            name: "猛攻の気迫".to_string(),
            description: "戦闘開始時、ダメージ1.3倍（被ダメも1.2倍）".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![
                SkillEffect {
                    effect_type: SkillEffectType::DamageMultiply,
                    target: SkillTarget::AllyUnit,
                    value: 1.3,
                    duration: None,
                },
                SkillEffect {
                    effect_type: SkillEffectType::DamageReduce,
                    target: SkillTarget::AllyUnit,
                    value: -0.2,
                    duration: None,
                },
            ],
            probability: 100,
        }),
        "steel_guard" => Some(Skill {
            id: "steel_guard".to_string(),
            name: "鋼の守り".to_string(),
            description: "戦闘開始時、被ダメージ20%軽減".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageReduce,
                target: SkillTarget::SelfOnly,
                value: 0.2,
                duration: None,
            }],
            probability: 100,
        }),
        "iron_fortress" => Some(Skill {
            id: "iron_fortress".to_string(),
            name: "鉄壁の陣".to_string(),
            description: "戦闘開始時、味方全員の被ダメージ10%軽減".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageReduce,
                target: SkillTarget::AllyUnit,
                value: 0.1,
                duration: None,
            }],
            probability: 100,
        }),
        "barrier_field" => Some(Skill {
            id: "barrier_field".to_string(),
            name: "結界展開".to_string(),
            description: "戦闘開始時、味方全員にシールド5付与".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Shield,
                target: SkillTarget::AllyUnit,
                value: 5.0,
                duration: None,
            }],
            probability: 100,
        }),
        "thorns" => Some(Skill {
            id: "thorns".to_string(),
            name: "荊の鎧".to_string(),
            description: "戦闘開始時、受けたダメージの20%を反射".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageReflect,
                target: SkillTarget::SelfOnly,
                value: 0.2,
                duration: Some(EffectDuration { turns: Some(99), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "counter_stance" => Some(Skill {
            id: "counter_stance".to_string(),
            name: "反撃の構え".to_string(),
            description: "戦闘開始時、反撃態勢（50%で反撃）".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Counter,
                target: SkillTarget::SelfOnly,
                value: 0.5,
                duration: Some(EffectDuration { turns: Some(99), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "shadow_veil" => Some(Skill {
            id: "shadow_veil".to_string(),
            name: "影の帳".to_string(),
            description: "戦闘開始時、回避率+20%".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Evasion,
                target: SkillTarget::SelfOnly,
                value: 0.2,
                duration: Some(EffectDuration { turns: Some(99), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "mist_cloak" => Some(Skill {
            id: "mist_cloak".to_string(),
            name: "霧隠れ".to_string(),
            description: "戦闘開始時、隠密状態（最初の攻撃を回避）".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Stealth,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "poison_aura" => Some(Skill {
            id: "poison_aura".to_string(),
            name: "瘴気の纏い".to_string(),
            description: "戦闘開始時、敵全員に毒（毎ターン2ダメージ）".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Poison,
                target: SkillTarget::EnemyAll,
                value: 2.0,
                duration: Some(EffectDuration { turns: Some(3), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "freezing_presence" => Some(Skill {
            id: "freezing_presence".to_string(),
            name: "凍てつく威圧".to_string(),
            description: "戦闘開始時、30%で敵全員を凍結".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Freeze,
                target: SkillTarget::EnemyAll,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 30,
        }),
        "intimidate" => Some(Skill {
            id: "intimidate".to_string(),
            name: "威圧".to_string(),
            description: "戦闘開始時、敵全員のダメージ15%低下".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Weaken,
                target: SkillTarget::EnemyAll,
                value: 0.15,
                duration: Some(EffectDuration { turns: Some(2), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "last_stand" => Some(Skill {
            id: "last_stand".to_string(),
            name: "背水の陣".to_string(),
            description: "HP50%以下で攻撃力2倍".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::HpLow,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::SelfOnly,
                value: 2.0,
                duration: None,
            }],
            probability: 100,
        }),
        "first_strike" => Some(Skill {
            id: "first_strike".to_string(),
            name: "先制の心得".to_string(),
            description: "初回攻撃時、ダメージ1.5倍".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::FirstAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::SelfOnly,
                value: 1.5,
                duration: None,
            }],
            probability: 100,
        }),
        "undying_will" => Some(Skill {
            id: "undying_will".to_string(),
            name: "不屈の意志".to_string(),
            description: "致死ダメージを1回だけHP1で耐える".to_string(),
            category: SkillCategory::Passive,
            timing: SkillTiming::OnDeath,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergySet,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),

        // === アクティブスキル ===
        // ダメージ強化系
        "critical_edge" => Some(Skill {
            id: "critical_edge".to_string(),
            name: "会心撃".to_string(),
            description: "攻撃時、30%でダメージ1.5倍".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::EnemySingle,
                value: 1.5,
                duration: None,
            }],
            probability: 30,
        }),
        "power_smash" => Some(Skill {
            id: "power_smash".to_string(),
            name: "剛撃".to_string(),
            description: "攻撃時、+5ダメージ".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageAdd,
                target: SkillTarget::EnemySingle,
                value: 5.0,
                duration: None,
            }],
            probability: 100,
        }),
        "flash_cut" => Some(Skill {
            id: "flash_cut".to_string(),
            name: "閃光斬".to_string(),
            description: "攻撃時、40%でダメージ1.2倍".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::EnemySingle,
                value: 1.2,
                duration: None,
            }],
            probability: 40,
        }),
        "heavy_impact" => Some(Skill {
            id: "heavy_impact".to_string(),
            name: "重衝撃".to_string(),
            description: "攻撃時、ダメージ1.1倍".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::EnemySingle,
                value: 1.1,
                duration: None,
            }],
            probability: 100,
        }),
        "sharp_thrust" => Some(Skill {
            id: "sharp_thrust".to_string(),
            name: "鋭突".to_string(),
            description: "攻撃時、+2ダメージ".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageAdd,
                target: SkillTarget::EnemySingle,
                value: 2.0,
                duration: None,
            }],
            probability: 100,
        }),
        "swift_blade" => Some(Skill {
            id: "swift_blade".to_string(),
            name: "迅刃".to_string(),
            description: "攻撃時、SPEED×0.5を追加ダメージ".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageAdd,
                target: SkillTarget::EnemySingle,
                value: 0.5,
                duration: None,
            }],
            probability: 100,
        }),
        // 複数攻撃・追加攻撃系
        "twin_strike" => Some(Skill {
            id: "twin_strike".to_string(),
            name: "双連撃".to_string(),
            description: "攻撃時、20%で追加攻撃".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::ExtraAttack,
                target: SkillTarget::EnemySingle,
                value: 1.0,
                duration: None,
            }],
            probability: 20,
        }),
        "triple_slash" => Some(Skill {
            id: "triple_slash".to_string(),
            name: "三連斬".to_string(),
            description: "攻撃時、10%で2回追加攻撃".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::ExtraAttack,
                target: SkillTarget::EnemySingle,
                value: 2.0,
                duration: None,
            }],
            probability: 10,
        }),
        "whirlwind" => Some(Skill {
            id: "whirlwind".to_string(),
            name: "旋風撃".to_string(),
            description: "攻撃時、敵全体に3ダメージ".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::TrueDamage,
                target: SkillTarget::EnemyAll,
                value: 3.0,
                duration: None,
            }],
            probability: 100,
        }),
        // 吸収・回復系
        "life_drain" => Some(Skill {
            id: "life_drain".to_string(),
            name: "生命吸収".to_string(),
            description: "攻撃時、与ダメージの30%を回復".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Absorb,
                target: SkillTarget::SelfOnly,
                value: 0.3,
                duration: None,
            }],
            probability: 100,
        }),
        "energy_steal" => Some(Skill {
            id: "energy_steal".to_string(),
            name: "奪命の一撃".to_string(),
            description: "攻撃時、敵から3エナジーを奪う".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergySteal,
                target: SkillTarget::EnemySingle,
                value: 3.0,
                duration: None,
            }],
            probability: 100,
        }),
        "heal_strike" => Some(Skill {
            id: "heal_strike".to_string(),
            name: "癒しの剣".to_string(),
            description: "攻撃時、HP最低の味方を3回復".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Heal,
                target: SkillTarget::AllyLowestHp,
                value: 3.0,
                duration: None,
            }],
            probability: 100,
        }),
        // 防御無視・特殊ダメージ系
        "armor_break" => Some(Skill {
            id: "armor_break".to_string(),
            name: "破甲撃".to_string(),
            description: "攻撃時、敵の防御を無視".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::TrueDamage,
                target: SkillTarget::EnemySingle,
                value: 0.0,
                duration: None,
            }],
            probability: 100,
        }),
        "percent_cut" => Some(Skill {
            id: "percent_cut".to_string(),
            name: "割合斬り".to_string(),
            description: "攻撃時、敵の現在HPの15%ダメージ".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::PercentDamage,
                target: SkillTarget::EnemySingle,
                value: 0.15,
                duration: None,
            }],
            probability: 100,
        }),
        "execute_blade" => Some(Skill {
            id: "execute_blade".to_string(),
            name: "処刑剣".to_string(),
            description: "攻撃時、敵HP30%以下で即死".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Execute,
                target: SkillTarget::EnemySingle,
                value: 0.3,
                duration: None,
            }],
            probability: 100,
        }),
        // 状態異常付与系
        "blaze_edge" => Some(Skill {
            id: "blaze_edge".to_string(),
            name: "炎刃".to_string(),
            description: "攻撃時、敵に炎上付与（3ターン、毎ターン3ダメージ）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Burn,
                target: SkillTarget::EnemySingle,
                value: 3.0,
                duration: Some(EffectDuration { turns: Some(3), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "venom_fang" => Some(Skill {
            id: "venom_fang".to_string(),
            name: "毒牙".to_string(),
            description: "攻撃時、敵に毒付与（3ターン、毎ターン2ダメージ）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Poison,
                target: SkillTarget::EnemySingle,
                value: 2.0,
                duration: Some(EffectDuration { turns: Some(3), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "frost_blade" => Some(Skill {
            id: "frost_blade".to_string(),
            name: "氷刃".to_string(),
            description: "攻撃時、25%で敵を凍結（1ターン行動不能）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Freeze,
                target: SkillTarget::EnemySingle,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 25,
        }),
        "thunder_strike" => Some(Skill {
            id: "thunder_strike".to_string(),
            name: "雷撃".to_string(),
            description: "攻撃時、30%で敵を気絶（1ターン行動不能）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Stun,
                target: SkillTarget::EnemySingle,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 30,
        }),
        "curse_touch" => Some(Skill {
            id: "curse_touch".to_string(),
            name: "呪縛の手".to_string(),
            description: "攻撃時、敵を脆弱化（被ダメージ20%増加、2ターン）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Vulnerable,
                target: SkillTarget::EnemySingle,
                value: 0.2,
                duration: Some(EffectDuration { turns: Some(2), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "silence_cut" => Some(Skill {
            id: "silence_cut".to_string(),
            name: "封魔斬".to_string(),
            description: "攻撃時、40%で敵を沈黙（スキル使用不可、2ターン）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Silence,
                target: SkillTarget::EnemySingle,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(2), stacks: None, max_stacks: None }),
            }],
            probability: 40,
        }),
        // バフ付与系
        "battle_cry" => Some(Skill {
            id: "battle_cry".to_string(),
            name: "鼓舞".to_string(),
            description: "攻撃時、味方全員の攻撃力+10%（2ターン）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::AttackBuff,
                target: SkillTarget::AllyUnit,
                value: 0.1,
                duration: Some(EffectDuration { turns: Some(2), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "shield_bash" => Some(Skill {
            id: "shield_bash".to_string(),
            name: "盾撃".to_string(),
            description: "攻撃時、自分にシールド3付与".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Shield,
                target: SkillTarget::SelfOnly,
                value: 3.0,
                duration: None,
            }],
            probability: 100,
        }),
        // デバフ解除・バフ解除系
        "purify_strike" => Some(Skill {
            id: "purify_strike".to_string(),
            name: "浄化の一撃".to_string(),
            description: "攻撃時、自分のデバフを1つ解除".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Cleanse,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),
        "dispel_blow" => Some(Skill {
            id: "dispel_blow".to_string(),
            name: "破魔撃".to_string(),
            description: "攻撃時、敵のバフを1つ解除".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Dispel,
                target: SkillTarget::EnemySingle,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),
        // 特殊系
        "mark_target" => Some(Skill {
            id: "mark_target".to_string(),
            name: "狙撃".to_string(),
            description: "攻撃時、敵にマーク付与（被ダメージ+5、3ターン）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Mark,
                target: SkillTarget::EnemySingle,
                value: 5.0,
                duration: Some(EffectDuration { turns: Some(3), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "taunt_blow" => Some(Skill {
            id: "taunt_blow".to_string(),
            name: "挑発撃".to_string(),
            description: "攻撃時、自分に挑発付与（敵の攻撃を引きつける）".to_string(),
            category: SkillCategory::Active,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Taunt,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(2), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),

        // === ユニークスキル ===
        // 攻撃強化系
        "divine_spear" => Some(Skill {
            id: "divine_spear".to_string(),
            name: "神槍・絶命".to_string(),
            description: "攻撃時、ダメージ2倍".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageMultiply,
                target: SkillTarget::EnemySingle,
                value: 2.0,
                duration: None,
            }],
            probability: 100,
        }),
        "thunder_call" => Some(Skill {
            id: "thunder_call".to_string(),
            name: "雷神招来".to_string(),
            description: "攻撃時、50%で敵全体に+10ダメージ".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::TrueDamage,
                target: SkillTarget::EnemyAll,
                value: 10.0,
                duration: None,
            }],
            probability: 50,
        }),
        // 蘇生・不死系
        "fake_death" => Some(Skill {
            id: "fake_death".to_string(),
            name: "偽りの死".to_string(),
            description: "死亡時、50%でエナジー半分で復活".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnDeath,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Revive,
                target: SkillTarget::SelfOnly,
                value: 0.5,
                duration: None,
            }],
            probability: 50,
        }),
        "phoenix_rebirth" => Some(Skill {
            id: "phoenix_rebirth".to_string(),
            name: "不死鳥の転生".to_string(),
            description: "死亡時、100%でエナジー全快で復活（1回のみ）".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnDeath,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Revive,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),
        // バフ系
        "beauty_charm" => Some(Skill {
            id: "beauty_charm".to_string(),
            name: "美神の輝き".to_string(),
            description: "戦闘開始時、味方全員のエナジー1.5倍".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergyMultiply,
                target: SkillTarget::AllyUnit,
                value: 1.5,
                duration: None,
            }],
            probability: 100,
        }),
        "war_god_blessing" => Some(Skill {
            id: "war_god_blessing".to_string(),
            name: "軍神の加護".to_string(),
            description: "戦闘開始時、味方全員に無敵（1回攻撃無効）".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Invincible,
                target: SkillTarget::AllyUnit,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),
        // 即死・処刑系
        "death_sentence" => Some(Skill {
            id: "death_sentence".to_string(),
            name: "死刑宣告".to_string(),
            description: "攻撃時、敵HP50%以下で即死".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Execute,
                target: SkillTarget::EnemySingle,
                value: 0.5,
                duration: None,
            }],
            probability: 100,
        }),
        "soul_reap" => Some(Skill {
            id: "soul_reap".to_string(),
            name: "魂狩り".to_string(),
            description: "敵を倒した時、その敵のエナジーの50%を獲得".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnKill,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::EnergySteal,
                target: SkillTarget::EnemySingle,
                value: 0.5,
                duration: None,
            }],
            probability: 100,
        }),
        // 状態異常系
        "absolute_zero" => Some(Skill {
            id: "absolute_zero".to_string(),
            name: "絶対零度".to_string(),
            description: "戦闘開始時、敵全員を凍結（1ターン行動不能）".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Freeze,
                target: SkillTarget::EnemyAll,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "plague_touch" => Some(Skill {
            id: "plague_touch".to_string(),
            name: "疫病の手".to_string(),
            description: "攻撃時、敵全員に毒付与（毎ターン5ダメージ、3ターン）".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnAttack,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::Poison,
                target: SkillTarget::EnemyAll,
                value: 5.0,
                duration: Some(EffectDuration { turns: Some(3), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        // 特殊系
        "time_stop" => Some(Skill {
            id: "time_stop".to_string(),
            name: "時間停止".to_string(),
            description: "戦闘開始時、味方全員が2回行動".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::ExtraAttack,
                target: SkillTarget::AllyUnit,
                value: 1.0,
                duration: None,
            }],
            probability: 100,
        }),
        "mirror_force" => Some(Skill {
            id: "mirror_force".to_string(),
            name: "鏡像反転".to_string(),
            description: "戦闘開始時、受けたダメージを100%反射（1回）".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![SkillEffect {
                effect_type: SkillEffectType::DamageReflect,
                target: SkillTarget::SelfOnly,
                value: 1.0,
                duration: Some(EffectDuration { turns: Some(1), stacks: None, max_stacks: None }),
            }],
            probability: 100,
        }),
        "dimensional_rift" => Some(Skill {
            id: "dimensional_rift".to_string(),
            name: "次元断裂".to_string(),
            description: "攻撃時、敵のバフを全て解除し、自分にコピー".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::OnAttack,
            effects: vec![
                SkillEffect {
                    effect_type: SkillEffectType::Dispel,
                    target: SkillTarget::EnemySingle,
                    value: 99.0,
                    duration: None,
                },
                SkillEffect {
                    effect_type: SkillEffectType::CopyBuff,
                    target: SkillTarget::SelfOnly,
                    value: 1.0,
                    duration: None,
                },
            ],
            probability: 100,
        }),
        "sacrifice" => Some(Skill {
            id: "sacrifice".to_string(),
            name: "生贄の儀".to_string(),
            description: "自分のHP半分を消費し、味方全員のエナジー2倍".to_string(),
            category: SkillCategory::Unique,
            timing: SkillTiming::BattleStart,
            effects: vec![
                SkillEffect {
                    effect_type: SkillEffectType::PercentDamage,
                    target: SkillTarget::SelfOnly,
                    value: 0.5,
                    duration: None,
                },
                SkillEffect {
                    effect_type: SkillEffectType::EnergyMultiply,
                    target: SkillTarget::AllyUnit,
                    value: 2.0,
                    duration: None,
                },
            ],
            probability: 100,
        }),

        _ => None,
    }
}

/// スキルの発動判定
pub fn should_trigger_skill(probability: u8) -> bool {
    if probability >= 100 {
        return true;
    }
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100) < probability
}

/// 戦闘開始時のパッシブスキル適用
pub fn apply_battle_start_skills(
    characters: &mut [CombatCharacter],
    log: &mut Vec<String>,
) {
    let skill_ids: Vec<(usize, Option<String>, Option<String>)> = characters
        .iter()
        .map(|c| (c.index, c.skills.passive_id.clone(), c.skills.unique_id.clone()))
        .collect();

    for (char_idx, passive_id, unique_id) in skill_ids {
        if let Some(id) = passive_id {
            if let Some(skill) = get_skill(&id) {
                if skill.timing == SkillTiming::BattleStart && should_trigger_skill(skill.probability) {
                    let char_name = characters.iter().find(|c| c.index == char_idx).map(|c| c.name.clone()).unwrap_or_default();
                    log.push(format!("◆ {}の「{}」が発動！", char_name, skill.name));
                    apply_skill_effects(&skill, char_idx, characters, log);
                }
            }
        }

        if let Some(id) = unique_id {
            if let Some(skill) = get_skill(&id) {
                if skill.timing == SkillTiming::BattleStart && should_trigger_skill(skill.probability) {
                    let char_name = characters.iter().find(|c| c.index == char_idx).map(|c| c.name.clone()).unwrap_or_default();
                    log.push(format!("◆◆ {}の固有スキル「{}」が発動！", char_name, skill.name));
                    apply_skill_effects(&skill, char_idx, characters, log);
                }
            }
        }
    }
}

/// 攻撃時のスキル適用（アクティブ・ユニーク）
pub fn apply_attack_skills(
    attacker: &mut CombatCharacter,
    log: &mut Vec<String>,
) -> AttackModifiers {
    let mut modifiers = AttackModifiers::default();

    if let Some(skill) = get_skill(&attacker.skills.active_id) {
        if skill.timing == SkillTiming::OnAttack && should_trigger_skill(skill.probability) {
            log.push(format!("★ {}の「{}」が発動！", attacker.name, skill.name));
            for effect in &skill.effects {
                apply_attack_effect(effect, attacker, &mut modifiers, log);
            }
        }
    }

    if let Some(ref unique_id) = attacker.skills.unique_id {
        if let Some(skill) = get_skill(unique_id) {
            if skill.timing == SkillTiming::OnAttack && should_trigger_skill(skill.probability) {
                log.push(format!("★★ {}の固有スキル「{}」が発動！", attacker.name, skill.name));
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
    pub damage_multiplier: f32,
    pub damage_add: f32,
    pub extra_attacks: u32,
    pub absorb_rate: f32,
    pub ignore_defense: bool,
    pub aoe_damage: f32,
    pub true_damage: f32,
    pub percent_damage: f32,
    pub execute_threshold: f32,
    pub energy_steal: f32,
    pub status_effects: Vec<SkillEffect>,
    pub self_effects: Vec<SkillEffect>,
    pub ally_effects: Vec<SkillEffect>,
    pub heal_effects: Vec<SkillEffect>,
}

impl AttackModifiers {
    pub fn new() -> Self {
        Self {
            damage_multiplier: 1.0,
            damage_add: 0.0,
            extra_attacks: 0,
            absorb_rate: 0.0,
            ignore_defense: false,
            aoe_damage: 0.0,
            true_damage: 0.0,
            percent_damage: 0.0,
            execute_threshold: 0.0,
            energy_steal: 0.0,
            status_effects: Vec::new(),
            self_effects: Vec::new(),
            ally_effects: Vec::new(),
            heal_effects: Vec::new(),
        }
    }
}

fn apply_skill_effects(
    skill: &Skill,
    caster_idx: usize,
    characters: &mut [CombatCharacter],
    log: &mut Vec<String>,
) {
    for effect in &skill.effects {
        match effect.target {
            SkillTarget::SelfOnly => {
                if let Some(c) = characters.iter_mut().find(|c| c.index == caster_idx) {
                    apply_effect_to_character(effect, c, log);
                }
            }
            SkillTarget::AllyUnit => {
                for c in characters.iter_mut() {
                    apply_effect_to_character(effect, c, log);
                }
            }
            _ => {}
        }
    }
}

pub fn apply_effect_to_character(
    effect: &SkillEffect,
    character: &mut CombatCharacter,
    log: &mut Vec<String>,
) {
    let turns = effect.duration.as_ref().and_then(|d| d.turns).unwrap_or(99);
    
    match effect.effect_type {
        // ステータス変更系
        SkillEffectType::EnergyMultiply => {
            let before = character.current_energy;
            character.current_energy *= effect.value;
            log.push(format!(
                "  → {}のエナジー: {:.0} → {:.0}",
                character.name, before, character.current_energy
            ));
        }
        SkillEffectType::EnergyAdd => {
            let before = character.current_energy;
            character.current_energy += effect.value;
            log.push(format!(
                "  → {}のエナジー: {:.0} → {:.0}",
                character.name, before, character.current_energy
            ));
        }
        SkillEffectType::EnergySet => {
            let before = character.current_energy;
            character.current_energy = effect.value;
            log.push(format!(
                "  → {}のエナジー: {:.0} → {:.0}",
                character.name, before, character.current_energy
            ));
        }
        SkillEffectType::SpeedMultiply => {
            let before = character.current_speed;
            character.current_speed *= effect.value;
            log.push(format!(
                "  → {}のSPEED: {:.0} → {:.0}",
                character.name, before, character.current_speed
            ));
        }
        SkillEffectType::SpeedAdd => {
            let before = character.current_speed;
            character.current_speed += effect.value;
            log.push(format!(
                "  → {}のSPEED: {:.0} → {:.0}",
                character.name, before, character.current_speed
            ));
        }
        SkillEffectType::DamageMultiply => {
            character.damage_multiplier *= effect.value;
            log.push(format!(
                "  → {}のダメージ倍率: x{:.1}",
                character.name, character.damage_multiplier
            ));
        }
        SkillEffectType::DamageReduce => {
            character.damage_reduction += effect.value;
            if effect.value > 0.0 {
                log.push(format!(
                    "  → {}の被ダメージ軽減: {:.0}%",
                    character.name, character.damage_reduction * 100.0
                ));
            } else {
                log.push(format!(
                    "  → {}の被ダメージ増加: +{:.0}%",
                    character.name, -effect.value * 100.0
                ));
            }
        }
        SkillEffectType::DamageReflect => {
            character.status_effects.push(StatusEffect::DamageReflect { rate: effect.value, turns });
            log.push(format!("  → {}にダメージ反射{:.0}%付与", character.name, effect.value * 100.0));
        }
        // 防御系
        SkillEffectType::Shield => {
            character.status_effects.push(StatusEffect::Shield { amount: effect.value });
            log.push(format!("  → {}にシールド{:.0}付与", character.name, effect.value));
        }
        SkillEffectType::Invincible => {
            character.status_effects.push(StatusEffect::Invincible { count: effect.value as u8 });
            log.push(format!("  → {}に無敵（{:.0}回）付与", character.name, effect.value));
        }
        SkillEffectType::Evasion => {
            character.status_effects.push(StatusEffect::Evasion { rate: effect.value, turns });
            log.push(format!("  → {}に回避率+{:.0}%付与", character.name, effect.value * 100.0));
        }
        SkillEffectType::Counter => {
            character.status_effects.push(StatusEffect::Counter { rate: effect.value, turns });
            log.push(format!("  → {}に反撃態勢（{:.0}%）付与", character.name, effect.value * 100.0));
        }
        // 状態異常系
        SkillEffectType::Poison => {
            character.status_effects.push(StatusEffect::Poison { damage: effect.value, turns });
            log.push(format!("  → {}に毒付与（毎ターン{:.0}ダメージ）", character.name, effect.value));
        }
        SkillEffectType::Burn => {
            character.status_effects.push(StatusEffect::Burn { damage: effect.value, turns });
            log.push(format!("  → {}に炎上付与（毎ターン{:.0}ダメージ）", character.name, effect.value));
        }
        SkillEffectType::Freeze => {
            character.status_effects.push(StatusEffect::Freeze { turns });
            log.push(format!("  → {}を凍結！（{}ターン行動不能）", character.name, turns));
        }
        SkillEffectType::Stun => {
            character.status_effects.push(StatusEffect::Stun { turns });
            log.push(format!("  → {}を気絶させた！（{}ターン行動不能）", character.name, turns));
        }
        SkillEffectType::Silence => {
            character.status_effects.push(StatusEffect::Silence { turns });
            log.push(format!("  → {}を沈黙！（{}ターンスキル使用不可）", character.name, turns));
        }
        SkillEffectType::Blind => {
            character.status_effects.push(StatusEffect::Blind { miss_rate: effect.value, turns });
            log.push(format!("  → {}に暗闘付与（命中率{:.0}%低下）", character.name, effect.value * 100.0));
        }
        SkillEffectType::Weaken => {
            character.status_effects.push(StatusEffect::Weaken { reduction: effect.value, turns });
            log.push(format!("  → {}に弱体化付与（ダメージ{:.0}%低下）", character.name, effect.value * 100.0));
        }
        SkillEffectType::Vulnerable => {
            character.status_effects.push(StatusEffect::Vulnerable { increase: effect.value, turns });
            log.push(format!("  → {}に脆弱付与（被ダメージ{:.0}%増加）", character.name, effect.value * 100.0));
        }
        SkillEffectType::Mark => {
            character.status_effects.push(StatusEffect::Mark { bonus_damage: effect.value, turns });
            log.push(format!("  → {}にマーク付与（被ダメージ+{:.0}）", character.name, effect.value));
        }
        // バフ系
        SkillEffectType::AttackBuff => {
            character.status_effects.push(StatusEffect::AttackBuff { bonus: effect.value, turns });
            log.push(format!("  → {}に攻撃バフ+{:.0}%付与", character.name, effect.value * 100.0));
        }
        SkillEffectType::DefenseBuff => {
            character.status_effects.push(StatusEffect::DefenseBuff { bonus: effect.value, turns });
            log.push(format!("  → {}に防御バフ+{:.0}%付与", character.name, effect.value * 100.0));
        }
        SkillEffectType::SpeedBuff => {
            character.status_effects.push(StatusEffect::SpeedBuff { bonus: effect.value, turns });
            log.push(format!("  → {}に速度バフ+{:.0}%付与", character.name, effect.value * 100.0));
        }
        // 特殊系
        SkillEffectType::Taunt => {
            character.status_effects.push(StatusEffect::Taunt { turns });
            log.push(format!("  → {}に挑発付与（敵の攻撃を引きつける）", character.name));
        }
        SkillEffectType::Stealth => {
            character.status_effects.push(StatusEffect::Stealth { turns });
            log.push(format!("  → {}が隠密状態に", character.name));
        }
        SkillEffectType::ExtraAttack => {
            character.extra_attacks += effect.value as u32;
            log.push(format!("  → {}に追加攻撃{:.0}回付与", character.name, effect.value));
        }
        SkillEffectType::Heal => {
            let before = character.current_energy;
            character.current_energy = (character.current_energy + effect.value).min(character.base_energy as f32);
            log.push(format!(
                "  → {}を{:.0}回復（{:.0} → {:.0}）",
                character.name, effect.value, before, character.current_energy
            ));
        }
        SkillEffectType::HealPercent => {
            let before = character.current_energy;
            let heal_amount = character.base_energy as f32 * effect.value;
            character.current_energy = (character.current_energy + heal_amount).min(character.base_energy as f32);
            log.push(format!(
                "  → {}を{:.0}%回復（{:.0} → {:.0}）",
                character.name, effect.value * 100.0, before, character.current_energy
            ));
        }
        SkillEffectType::PercentDamage => {
            let damage = character.current_energy * effect.value;
            character.current_energy -= damage;
            log.push(format!(
                "  → {}に{:.0}%ダメージ（{:.0}）",
                character.name, effect.value * 100.0, damage
            ));
            if character.current_energy <= 0.0 {
                character.is_alive = false;
                log.push(format!("  → {}が倒れた！", character.name));
            }
        }
        SkillEffectType::Cleanse => {
            let count = effect.value as usize;
            let mut removed = 0;
            character.status_effects.retain(|e| {
                if removed >= count { return true; }
                match e {
                    StatusEffect::Poison { .. } | StatusEffect::Burn { .. } |
                    StatusEffect::Freeze { .. } | StatusEffect::Stun { .. } |
                    StatusEffect::Silence { .. } | StatusEffect::Blind { .. } |
                    StatusEffect::Weaken { .. } | StatusEffect::Vulnerable { .. } |
                    StatusEffect::Mark { .. } => {
                        removed += 1;
                        false
                    }
                    _ => true
                }
            });
            if removed > 0 {
                log.push(format!("  → {}のデバフを{}個解除", character.name, removed));
            }
        }
        SkillEffectType::Dispel => {
            let count = effect.value as usize;
            let mut removed = 0;
            character.status_effects.retain(|e| {
                if removed >= count { return true; }
                match e {
                    StatusEffect::AttackBuff { .. } | StatusEffect::DefenseBuff { .. } |
                    StatusEffect::SpeedBuff { .. } | StatusEffect::Shield { .. } |
                    StatusEffect::Invincible { .. } | StatusEffect::Evasion { .. } |
                    StatusEffect::Counter { .. } | StatusEffect::DamageReflect { .. } |
                    StatusEffect::Stealth { .. } => {
                        removed += 1;
                        false
                    }
                    _ => true
                }
            });
            if removed > 0 {
                log.push(format!("  → {}のバフを{}個解除", character.name, removed));
            }
        }
        _ => {}
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
            log.push(format!("  → ダメージ x{:.1}倍", effect.value));
        }
        SkillEffectType::DamageAdd => {
            if effect.target == SkillTarget::EnemySingle && effect.value < 1.0 {
                let bonus = attacker.current_speed * effect.value;
                modifiers.damage_add += bonus;
                log.push(format!("  → +{:.0}ダメージ（SPEED補正）", bonus));
            } else {
                modifiers.damage_add += effect.value;
                log.push(format!("  → +{:.0}ダメージ", effect.value));
            }
        }
        SkillEffectType::ExtraAttack => {
            modifiers.extra_attacks += effect.value as u32;
            log.push("  → 追加攻撃発生！".to_string());
        }
        SkillEffectType::Absorb => {
            modifiers.absorb_rate += effect.value;
            log.push(format!("  → 与ダメージの{:.0}%を吸収", effect.value * 100.0));
        }
        SkillEffectType::TrueDamage => {
            modifiers.true_damage += effect.value;
            modifiers.ignore_defense = true;
            if effect.value > 0.0 {
                log.push(format!("  → 固定ダメージ{:.0}追加", effect.value));
            } else {
                log.push("  → 防御無視".to_string());
            }
        }
        SkillEffectType::PercentDamage => {
            modifiers.percent_damage += effect.value;
            log.push(format!("  → 敵現在HPの{:.0}%追加ダメージ", effect.value * 100.0));
        }
        SkillEffectType::Execute => {
            modifiers.execute_threshold = modifiers.execute_threshold.max(effect.value);
            log.push(format!("  → HP{:.0}%以下の敵を即死", effect.value * 100.0));
        }
        SkillEffectType::EnergySteal => {
            modifiers.energy_steal += effect.value;
            log.push(format!("  → {:.0}エナジー奪取", effect.value));
        }
        // 状態異常付与系は別途処理
        SkillEffectType::Poison | SkillEffectType::Burn | SkillEffectType::Freeze |
        SkillEffectType::Stun | SkillEffectType::Silence | SkillEffectType::Vulnerable |
        SkillEffectType::Mark | SkillEffectType::Weaken => {
            modifiers.status_effects.push(effect.clone());
        }
        // バフ系
        SkillEffectType::Shield => {
            modifiers.self_effects.push(effect.clone());
            log.push(format!("  → シールド{:.0}付与", effect.value));
        }
        SkillEffectType::AttackBuff => {
            modifiers.ally_effects.push(effect.clone());
            log.push(format!("  → 味方に攻撃バフ+{:.0}%", effect.value * 100.0));
        }
        SkillEffectType::Heal => {
            modifiers.heal_effects.push(effect.clone());
            log.push(format!("  → {:.0}回復効果", effect.value));
        }
        SkillEffectType::Cleanse | SkillEffectType::Dispel => {
            modifiers.self_effects.push(effect.clone());
        }
        SkillEffectType::Taunt => {
            modifiers.self_effects.push(effect.clone());
            log.push("  → 挑発付与".to_string());
        }
        _ => {}
    }

    if effect.target == SkillTarget::EnemyAll {
        modifiers.aoe_damage += effect.value;
    }
}

/// 死亡時スキル（復活など）のチェック
pub fn check_death_skills(
    character: &mut CombatCharacter,
    log: &mut Vec<String>,
) -> bool {
    if let Some(ref unique_id) = character.skills.unique_id {
        if let Some(skill) = get_skill(unique_id) {
            if skill.timing == SkillTiming::OnDeath && should_trigger_skill(skill.probability) {
                for effect in &skill.effects {
                    if effect.effect_type == SkillEffectType::Revive {
                        character.current_energy = character.base_energy as f32 * effect.value;
                        character.is_alive = true;
                        log.push(format!(
                            "{}の「{}」が発動！エナジー{}で復活！",
                            character.name,
                            skill.name,
                            character.effective_energy()
                        ));
                        return true;
                    }
                }
            }
        }
    }
    false
}
