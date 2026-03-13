//! 施設定義とボーナス計算

/// 施設ボーナス集計結果
#[derive(Debug, Clone, Default)]
pub struct FacilityBonuses {
    pub energy_bonus: u32,
    pub energy_percent: u32,
    pub speed_bonus: u32,
    pub skill_power: u32,
    pub drop_rate: u32,
    pub exp_bonus: u32,
    pub storage_capacity: u32,
    pub unit_capacity: u32,
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
            "energy_well" => {
                let values = [5, 10, 18, 28, 40];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.energy_bonus += v;
                }
            }
            "training_ground" => {
                let values = [10, 20, 35, 50, 70];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.energy_percent += v;
                }
            }
            "armory" => {
                let values = [2, 4, 7];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.speed_bonus += v;
                }
            }
            "magic_tower" => {
                let values = [10, 25, 45, 70];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.skill_power += v;
                }
            }
            "research_lab" => {
                let values = [20, 50, 100];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.exp_bonus += v;
                }
            }
            "watchtower" => {
                let values = [25, 50];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.drop_rate += v;
                }
            }
            "altar" => {
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
            "barracks" => {
                let values = [1, 2, 3];
                if let Some(&v) = values.get(facility.level as usize - 1) {
                    bonuses.unit_capacity += v;
                }
            }
            _ => {}
        }
    }
    
    bonuses
}

/// 施設ボーナスを適用したエナジーを計算
pub fn apply_energy_bonus(base_energy: u32, bonuses: &FacilityBonuses) -> u32 {
    let with_bonus = base_energy + bonuses.energy_bonus;
    let with_percent = with_bonus as f64 * (1.0 + bonuses.energy_percent as f64 / 100.0);
    with_percent as u32
}
