//! 施設定義とボーナス計算

/// 施設ボーナス集計結果
#[derive(Debug, Clone, Default)]
pub struct FacilityBonuses {
    pub monster_bonus: u32,
    pub monster_percent: u32,
    pub speed_bonus: u32,
    pub skill_power: u32,
    pub drop_rate: u32,
    pub exp_bonus: u32,
    pub storage_capacity: u32,
    pub unit_capacity: u32,
    pub market_fee_reduction: u32,
    pub defense_bonus: u32,
    pub attack_bonus: u32,
    /// 研究所系: ユニット編成コスト上限の加算（KC）
    pub unit_cost_cap_bonus: f32,
    /// 兵舎系: ターン終了時のスタミナ追加回復量
    pub stamina_recovery_bonus: u32,
}

use crate::model::BuiltFacility;

/// 建設済み施設からボーナスを集計
pub fn calculate_facility_bonuses(facilities: &[BuiltFacility]) -> FacilityBonuses {
    let mut bonuses = FacilityBonuses::default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    
    for facility in facilities {
        // 建設中（まだ完了していない）は効果なし
        if let Some(complete_at) = facility.build_complete_at {
            if complete_at > now {
                continue;
            }
        }
        
        match facility.facility_id.as_str() {
            "monster_well" | "monster_barracks" => {
                let values = [5, 10, 18, 28, 40];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.monster_bonus += v;
                }
            }
            "training_ground" | "training_tower" => {
                let values = [10, 20, 35, 50, 70];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.monster_percent += v;
                }
            }
            "armory" | "hero_statue" => {
                let values = [2, 4, 7];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.speed_bonus += v;
                }
            }
            "magic_tower" | "battle_lab" => {
                let values = [10, 25, 45, 70];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.skill_power += v;
                }
            }
            "research_lab" | "library" => {
                let values = [20, 50, 100];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.exp_bonus += v;
                }
                let cap_bumps = [0.15_f32, 0.35, 0.6];
                if let Some(&v) = cap_bumps.get(facility.level as usize - 1) {
                    bonuses.unit_cost_cap_bonus += v;
                }
            }
            "watchtower" => {
                let values = [25, 50];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.drop_rate += v;
                }
            }
            "altar" | "guardian_shrine" => {
                let values = [15, 35, 60];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.drop_rate += v;
                }
            }
            "warehouse" => {
                let values = [100, 300, 600];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.storage_capacity += v;
                }
            }
            "barracks" | "stronghold" => {
                let values = [1, 2, 3];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.unit_capacity += v;
                }
                let stam = [2_u32, 4, 6];
                if let Some(&v) = stam.get(facility.level as usize - 1) {
                    bonuses.stamina_recovery_bonus += v;
                }
            }
            "trading_post" => {
                let values = [2, 4, 6, 8, 10];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.market_fee_reduction += v;
                }
            }
            "fortress" => {
                let values = [5, 10, 15, 20, 30];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.defense_bonus += v;
                }
            }
            "war_god_shrine" => {
                let values = [5, 10, 20];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.attack_bonus += v;
                }
            }
            _ => {}
        }
    }
    
    bonuses
}

/// 施設ボーナスを適用した魔獣数（monster count）を計算
pub fn apply_monster_bonus(base_monster_count: u32, bonuses: &FacilityBonuses) -> u32 {
    let with_bonus = base_monster_count + bonuses.monster_bonus;
    let with_percent = with_bonus as f64 * (1.0 + bonuses.monster_percent as f64 / 100.0);
    with_percent as u32
}
