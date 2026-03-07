//! 施設定義とボーナス計算

use serde::{Deserialize, Serialize};

/// 施設の効果タイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FacilityEffect {
    #[serde(rename = "energy_bonus")]
    EnergyBonus { value: u32 },
    #[serde(rename = "energy_percent")]
    EnergyPercent { value: u32 },
    #[serde(rename = "speed_bonus")]
    SpeedBonus { value: u32 },
    #[serde(rename = "skill_power")]
    SkillPower { value: u32 },
    #[serde(rename = "drop_rate")]
    DropRate { value: u32 },
    #[serde(rename = "exp_bonus")]
    ExpBonus { value: u32 },
    #[serde(rename = "storage_capacity")]
    StorageCapacity { value: u32 },
    #[serde(rename = "unit_capacity")]
    UnitCapacity { value: u32 },
    #[serde(rename = "passive_energy_regen")]
    PassiveEnergyRegen { value: u32 },
    #[serde(rename = "resource_production")]
    ResourceProduction { resource_id: String, value: u32 },
}

/// 施設レベルごとの必要素材
#[derive(Debug, Clone)]
pub struct FacilityCost {
    pub item_id: &'static str,
    pub count: u32,
}

/// 施設レベル定義
#[derive(Debug, Clone)]
pub struct FacilityLevelDef {
    pub level: u8,
    pub effect: FacilityEffectDef,
    pub cost: &'static [FacilityCost],
    pub build_time: u32, // seconds
}

/// 施設効果定義（静的）
#[derive(Debug, Clone, Copy)]
pub enum FacilityEffectDef {
    EnergyBonus(u32),
    EnergyPercent(u32),
    SpeedBonus(u32),
    SkillPower(u32),
    DropRate(u32),
    ExpBonus(u32),
    StorageCapacity(u32),
    UnitCapacity(u32),
}

/// 施設定義
#[derive(Debug, Clone)]
pub struct FacilityDef {
    pub id: &'static str,
    pub name: &'static str,
    pub max_level: u8,
    pub levels: &'static [FacilityLevelDef],
}

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

/// 施設レベルを取得（未建設は0）
pub fn get_facility_level(facilities: &[BuiltFacility], facility_id: &str) -> u8 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    
    for facility in facilities {
        if facility.facility_id == facility_id {
            // 建設中はレベル0として扱う
            if let Some(complete_at) = facility.build_complete_at {
                if complete_at > now {
                    return 0;
                }
            }
            return facility.level;
        }
    }
    0
}

/// 施設のコスト取得（facility_id, level）
pub fn get_facility_cost(facility_id: &str, level: u8) -> Vec<(String, u32)> {
    match facility_id {
        "energy_well" => match level {
            1 => vec![("ancient_stone".into(), 20), ("rotten_wood".into(), 15)],
            2 => vec![("ancient_stone".into(), 50), ("mystic_crystal".into(), 10)],
            3 => vec![("ancient_stone".into(), 100), ("mystic_crystal".into(), 30), ("shining_magicstone".into(), 5)],
            4 => vec![("ancient_stone".into(), 200), ("shining_magicstone".into(), 20), ("guardian_core".into(), 2)],
            5 => vec![("ancient_stone".into(), 400), ("shining_magicstone".into(), 50), ("guardian_core".into(), 5), ("ancient_kings_seal".into(), 1)],
            _ => vec![],
        },
        "training_ground" => match level {
            1 => vec![("ancient_stone".into(), 25), ("reinforced_fiber".into(), 10)],
            2 => vec![("ancient_stone".into(), 60), ("refined_iron".into(), 20), ("reinforced_fiber".into(), 20)],
            3 => vec![("ancient_stone".into(), 120), ("shining_magicstone".into(), 10), ("ancient_blueprint".into(), 3)],
            4 => vec![("ancient_stone".into(), 200), ("shining_magicstone".into(), 25), ("guardian_core".into(), 2)],
            5 => vec![("ancient_stone".into(), 350), ("shining_magicstone".into(), 50), ("guardian_core".into(), 4), ("dragon_scale".into(), 1)],
            _ => vec![],
        },
        "armory" => match level {
            1 => vec![("refined_iron".into(), 20), ("rusty_gear".into(), 15)],
            2 => vec![("refined_iron".into(), 50), ("golden_gear".into(), 2), ("ancient_blueprint".into(), 2)],
            3 => vec![("refined_iron".into(), 100), ("golden_gear".into(), 5), ("guardian_core".into(), 2)],
            _ => vec![],
        },
        "magic_tower" => match level {
            1 => vec![("mystic_crystal".into(), 25), ("magic_shard".into(), 30)],
            2 => vec![("mystic_crystal".into(), 60), ("shining_magicstone".into(), 10)],
            3 => vec![("shining_magicstone".into(), 30), ("guardian_core".into(), 2), ("ancient_kings_seal".into(), 1)],
            4 => vec![("shining_magicstone".into(), 60), ("guardian_core".into(), 5), ("dragon_scale".into(), 2)],
            _ => vec![],
        },
        "warehouse" => match level {
            1 => vec![("rotten_wood".into(), 40), ("broken_brick".into(), 30)],
            2 => vec![("rotten_wood".into(), 100), ("refined_iron".into(), 20)],
            3 => vec![("rotten_wood".into(), 200), ("golden_gear".into(), 3)],
            _ => vec![],
        },
        "barracks" => match level {
            1 => vec![("ancient_stone".into(), 30), ("rotten_wood".into(), 20), ("broken_brick".into(), 15)],
            2 => vec![("ancient_stone".into(), 80), ("refined_iron".into(), 20), ("reinforced_fiber".into(), 15)],
            3 => vec![("ancient_stone".into(), 150), ("golden_gear".into(), 2), ("guardian_core".into(), 1)],
            _ => vec![],
        },
        "crystal_mine" => match level {
            1 => vec![("ancient_stone".into(), 40), ("rusty_gear".into(), 20)],
            2 => vec![("ancient_stone".into(), 100), ("refined_iron".into(), 30), ("ancient_blueprint".into(), 2)],
            3 => vec![("ancient_stone".into(), 200), ("golden_gear".into(), 3), ("guardian_core".into(), 1)],
            _ => vec![],
        },
        "lumber_mill" => match level {
            1 => vec![("rotten_wood".into(), 30), ("rusty_gear".into(), 10)],
            2 => vec![("rotten_wood".into(), 80), ("refined_iron".into(), 15)],
            3 => vec![("rotten_wood".into(), 150), ("golden_gear".into(), 2)],
            _ => vec![],
        },
        "research_lab" => match level {
            1 => vec![("ancient_blueprint".into(), 3), ("magic_shard".into(), 20)],
            2 => vec![("ancient_blueprint".into(), 8), ("mystic_crystal".into(), 30), ("shining_magicstone".into(), 5)],
            3 => vec![("ancient_blueprint".into(), 15), ("shining_magicstone".into(), 20), ("guardian_core".into(), 2)],
            _ => vec![],
        },
        "watchtower" => match level {
            1 => vec![("ancient_stone".into(), 35), ("rotten_wood".into(), 25)],
            2 => vec![("ancient_stone".into(), 80), ("mystic_crystal".into(), 15), ("ancient_blueprint".into(), 2)],
            _ => vec![],
        },
        "altar" => match level {
            1 => vec![("mystic_crystal".into(), 20), ("magic_shard".into(), 25)],
            2 => vec![("mystic_crystal".into(), 50), ("shining_magicstone".into(), 8)],
            3 => vec![("shining_magicstone".into(), 25), ("guardian_core".into(), 2), ("dragon_scale".into(), 1)],
            _ => vec![],
        },
        "skill_shrine" => match level {
            1 => vec![("ancient_stone".into(), 50), ("magic_shard".into(), 40), ("ancient_blueprint".into(), 2)],
            2 => vec![("shining_magicstone".into(), 20), ("guardian_core".into(), 2), ("ancient_kings_seal".into(), 1)],
            _ => vec![],
        },
        _ => vec![],
    }
}

/// 施設の建設時間取得（秒）
pub fn get_facility_build_time(facility_id: &str, level: u8) -> u32 {
    match facility_id {
        "energy_well" => match level { 1 => 60, 2 => 180, 3 => 600, 4 => 1800, 5 => 3600, _ => 0 },
        "training_ground" => match level { 1 => 90, 2 => 300, 3 => 900, 4 => 2400, 5 => 3600, _ => 0 },
        "armory" => match level { 1 => 120, 2 => 600, 3 => 1800, _ => 0 },
        "magic_tower" => match level { 1 => 180, 2 => 600, 3 => 1800, 4 => 3600, _ => 0 },
        "warehouse" => match level { 1 => 60, 2 => 300, 3 => 900, _ => 0 },
        "barracks" => match level { 1 => 120, 2 => 600, 3 => 1800, _ => 0 },
        "crystal_mine" => match level { 1 => 120, 2 => 600, 3 => 1800, _ => 0 },
        "lumber_mill" => match level { 1 => 60, 2 => 300, 3 => 900, _ => 0 },
        "research_lab" => match level { 1 => 180, 2 => 900, 3 => 2400, _ => 0 },
        "watchtower" => match level { 1 => 120, 2 => 600, _ => 0 },
        "altar" => match level { 1 => 180, 2 => 600, 3 => 1800, _ => 0 },
        "skill_shrine" => match level { 1 => 300, 2 => 1200, _ => 0 },
        _ => 0,
    }
}

/// 施設の最大レベル取得
pub fn get_facility_max_level(facility_id: &str) -> u8 {
    match facility_id {
        "energy_well" => 5,
        "training_ground" => 5,
        "armory" => 3,
        "magic_tower" => 4,
        "warehouse" => 3,
        "barracks" => 3,
        "crystal_mine" => 3,
        "lumber_mill" => 3,
        "research_lab" => 3,
        "watchtower" => 2,
        "altar" => 3,
        "skill_shrine" => 2,
        _ => 0,
    }
}
