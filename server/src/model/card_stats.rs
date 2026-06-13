//! 魔獣の実効ステータス（マスタ + 配分ボーナス + 施設 + 強化★）

use serde::{Deserialize, Serialize};

use crate::cards::{get_card, CardStats};
use crate::facilities::{calculate_facility_bonuses, FacilityBonuses};

/// レベルアップ等で振り分け可能なステータスボーナス（所持スロットごと）
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardStatBonuses {
    pub speed: u32,
    pub attack: u32,
    pub intelligence: u32,
    pub defense: u32,
    pub magic_defense: u32,
}

impl CardStatBonuses {
    pub fn total(&self) -> u32 {
        self.speed + self.attack + self.intelligence + self.defense + self.magic_defense
    }

    pub fn add_assignments(&mut self, delta: &CardStatBonuses) {
        self.speed = self.speed.saturating_add(delta.speed);
        self.attack = self.attack.saturating_add(delta.attack);
        self.intelligence = self.intelligence.saturating_add(delta.intelligence);
        self.defense = self.defense.saturating_add(delta.defense);
        self.magic_defense = self.magic_defense.saturating_add(delta.magic_defense);
    }
}

/// 知力スキル補正の基準値（マスタ未設定時のフォールバックと一致）
pub const REF_INTELLIGENCE: u32 = 5;

/// マスタ・配分ボーナス・施設・強化★を反映した戦闘用ステータス
pub fn effective_card_stats(
    card_id: u32,
    bonuses: &CardStatBonuses,
    facility_bonuses: &FacilityBonuses,
    enhanced: bool,
) -> CardStats {
    let base = get_card(card_id)
        .map(|c| c.stats.clone())
        .unwrap_or_default();
    let mut stats = CardStats {
        monster_count: base.monster_count,
        speed: base
            .speed
            .saturating_add(bonuses.speed)
            .saturating_add(facility_bonuses.speed_bonus),
        attack: base.attack.saturating_add(bonuses.attack),
        intelligence: base.intelligence.saturating_add(bonuses.intelligence),
        defense: base.defense.saturating_add(bonuses.defense),
        magic_defense: base.magic_defense.saturating_add(bonuses.magic_defense),
        range: base.range,
        cost: base.cost,
        occupation_power: base.occupation_power,
    };
    if enhanced {
        let mul = |v: u32| ((v as f32) * 1.10).round() as u32;
        stats.speed = mul(stats.speed);
        stats.attack = mul(stats.attack);
        stats.intelligence = mul(stats.intelligence);
        stats.defense = mul(stats.defense);
        stats.magic_defense = mul(stats.magic_defense);
        stats.monster_count = mul(stats.monster_count);
    }
    stats
}

/// 所持スロット列を owned_cards 長に揃える
pub fn ensure_card_stat_bonuses(player: &mut crate::model::PlayerData) {
    while player.card_stat_bonuses.len() < player.owned_cards.len() {
        player.card_stat_bonuses.push(CardStatBonuses::default());
    }
    player.card_stat_bonuses.truncate(player.owned_cards.len());
}

/// サーバー権威: マスタ + 配分ボーナス + 施設 + 強化★から戦闘用ステータスを再計算
pub fn resolve_authoritative_body_stats(
    player: &crate::model::PlayerData,
    owned_card_indices: &[usize],
    monsters_per_body: Option<&[u32]>,
) -> Vec<CardStats> {
    let facility = calculate_facility_bonuses(&player.facilities);
    owned_card_indices
        .iter()
        .enumerate()
        .map(|(body_i, &slot)| {
            let card_id = player.owned_cards.get(slot).copied().unwrap_or(0);
            let bonuses = player
                .card_stat_bonuses
                .get(slot)
                .cloned()
                .unwrap_or_default();
            let enhanced = player.card_enhanced.get(slot).copied().unwrap_or(false);
            let mut stats = effective_card_stats(card_id, &bonuses, &facility, enhanced);
            if let Some(counts) = monsters_per_body {
                if let Some(&mc) = counts.get(body_i) {
                    if mc > 0 {
                        stats.monster_count = mc;
                    }
                }
            }
            stats
        })
        .collect()
}
