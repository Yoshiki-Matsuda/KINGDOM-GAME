use super::types::*;

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
    /// 毎攻撃前に value 確率で味方を対象にする
    Confused { hit_own_team_chance: f32, turns: u8 },
    /// 常に味方を攻撃対象とみなす
    Charmed { turns: u8 },
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

/// 部隊配置（KC準拠: Front=前衛, Back=中衛, Leader=指揮官）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Front,
    Back,
    Leader,
}

/// 戦闘中のキャラクター状態（スキル適用後の修正値を含む）
#[derive(Debug, Clone)]
pub struct CombatCharacter {
    pub index: usize,
    pub name: String,
    pub base_monster_count: u32,
    pub current_monster_count: f32,
    pub current_speed: f32,
    /// スタートアップ前の速さ（ウェーブ間でスキル効果をリセットする際の基準）
    pub pre_startup_speed: f32,
    /// 当ウェーブのスタートアップで掛かった魔獣数倍率（ウェーブ間リセット用）
    pub startup_monster_factor: f32,
    /// 攻撃力（物理ダメージ）
    pub attack: u32,
    /// 知力（スキル効果に影響）
    pub intelligence: u32,
    /// 防御力（物理防御）
    pub defense: u32,
    /// 魔法防御力
    pub magic_defense: u32,
    pub skills: SkillData,
    pub damage_multiplier: f32,
    pub damage_reduction: f32,
    pub extra_attacks: u32,
    pub is_alive: bool,
    pub status_effects: Vec<StatusEffect>,
    /// 配置 (Front=前衛, Back=中衛, Leader=指揮官)
    pub position: Position,
    /// 射程 (1=近接, 2=中距離, 3=遠距離)
    pub range: u8,
    /// 種族（KC7種族）
    pub race: Option<crate::cards::Race>,
    /// 占拠力（勝利時の耐久削りに使用）
    pub occupation_power: u32,
}

impl CombatCharacter {
    pub fn new(index: usize, name: String, monster_count: u32, speed: u32, skills: SkillData) -> Self {
        Self {
            index,
            name,
            base_monster_count: monster_count,
            current_monster_count: monster_count as f32,
            current_speed: speed as f32,
            pre_startup_speed: speed as f32,
            startup_monster_factor: 1.0,
            attack: 5,
            intelligence: 5,
            defense: 3,
            magic_defense: 3,
            skills,
            damage_multiplier: 1.0,
            damage_reduction: 0.0,
            extra_attacks: 0,
            is_alive: true,
            status_effects: Vec::new(),
            position: Position::Front,
            range: 1,
            race: None,
            occupation_power: 0,
        }
    }

    /// 全ステータスを指定してキャラクター作成
    pub fn with_stats(
        index: usize,
        name: String,
        monster_count: u32,
        speed: u32,
        attack: u32,
        intelligence: u32,
        defense: u32,
        magic_defense: u32,
        skills: SkillData,
    ) -> Self {
        Self {
            index,
            name,
            base_monster_count: monster_count,
            current_monster_count: monster_count as f32,
            current_speed: speed as f32,
            pre_startup_speed: speed as f32,
            startup_monster_factor: 1.0,
            attack,
            intelligence,
            defense,
            magic_defense,
            skills,
            damage_multiplier: 1.0,
            damage_reduction: 0.0,
            extra_attacks: 0,
            is_alive: true,
            status_effects: Vec::new(),
            position: Position::Front,
            range: 1,
            race: None,
            occupation_power: 0,
        }
    }

    pub fn effective_monster_count(&self) -> u32 {
        (self.current_monster_count.max(0.0).round()) as u32
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

    /// 挑発状態かどうか
    pub fn is_taunting(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Taunt { .. }))
    }

    pub fn is_charmed(&self) -> bool {
        self.status_effects.iter().any(|e| matches!(e, StatusEffect::Charmed { .. }))
    }

    /// 混乱による「自陣を狙う」追加確率（0〜1、魅了は含めない）
    pub fn confused_own_team_chance(&self) -> f32 {
        self.status_effects
            .iter()
            .filter_map(|e| {
                if let StatusEffect::Confused { hit_own_team_chance, .. } = e {
                    Some(*hit_own_team_chance)
                } else {
                    None
                }
            })
            .sum::<f32>()
            .min(1.0)
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

    /// 行動順用の実効素早さ（速度バフを反映）
    pub fn turn_order_speed(&self) -> f32 {
        let buff: f32 = self.status_effects.iter().filter_map(|e| {
            if let StatusEffect::SpeedBuff { bonus, .. } = e {
                Some(*bonus)
            } else {
                None
            }
        }).sum();
        self.current_speed * (1.0 + buff)
    }

    /// 攻撃バフによる攻撃側倍率（1.0基準）
    pub fn attack_buff_multiplier(&self) -> f32 {
        1.0 + self
            .status_effects
            .iter()
            .filter_map(|e| {
                if let StatusEffect::AttackBuff { bonus, .. } = e {
                    Some(*bonus)
                } else {
                    None
                }
            })
            .sum::<f32>()
    }

    /// 防御バフによる防御側倍率（1.0基準）
    pub fn defense_buff_multiplier(&self) -> f32 {
        1.0 + self
            .status_effects
            .iter()
            .filter_map(|e| {
                if let StatusEffect::DefenseBuff { bonus, .. } = e {
                    Some(*bonus)
                } else {
                    None
                }
            })
            .sum::<f32>()
    }

    /// 弱体化による与ダメージ倍率（1.0基準）
    pub fn outgoing_damage_multiplier(&self) -> f32 {
        let weaken: f32 = self
            .status_effects
            .iter()
            .filter_map(|e| {
                if let StatusEffect::Weaken { reduction, .. } = e {
                    Some(*reduction)
                } else {
                    None
                }
            })
            .sum();
        (1.0 - weaken.min(0.95)).max(0.05)
    }

    /// ターン経過処理（毒・炎上ダメージ、持続ターン減少）
    pub fn process_turn_effects(&mut self, log: &mut Vec<crate::model::GameEvent>) {
        let mut damage_taken = 0.0;

        for effect in &self.status_effects {
            match effect {
                StatusEffect::Poison { damage, .. } => {
                    damage_taken += damage;
                    crate::model::push_skill_effect_event(log, &format!("  {}が毒で {:.0} ダメージ！", self.name, damage));
                }
                StatusEffect::Burn { damage, .. } => {
                    damage_taken += damage;
                    crate::model::push_skill_effect_event(log, &format!("  {}が炎上で {:.0} ダメージ！", self.name, damage));
                }
                _ => {}
            }
        }

        if damage_taken > 0.0 {
            self.current_monster_count -= damage_taken;
            if self.current_monster_count <= 0.0 {
                self.is_alive = false;
                crate::model::push_skill_effect_event(log, &format!("  {}が状態異常で倒れた！", self.name));
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
                StatusEffect::Confused { turns, .. } |
                StatusEffect::Charmed { turns } |
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
