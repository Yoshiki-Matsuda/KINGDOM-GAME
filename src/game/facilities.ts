/**
 * 本拠地施設の定義
 */

import type { InventoryItem } from "../shared/game-state";
import { getItemCount } from "./items";

/** 施設のカテゴリ */
export type FacilityCategory = "production" | "military" | "research" | "special";

/** 施設ID */
export type FacilityId =
  // 生産系
  | "energy_well"
  | "crystal_mine"
  | "lumber_mill"
  // 軍事系
  | "barracks"
  | "training_ground"
  | "armory"
  // 研究系
  | "research_lab"
  | "magic_tower"
  | "skill_shrine"
  // 特殊系
  | "warehouse"
  | "watchtower"
  | "altar"
  | "home_expansion";

/** 施設レベルごとの効果 */
export interface FacilityLevelEffect {
  level: number;
  description: string;
  effect: FacilityEffect;
  cost: { itemId: string; count: number }[];
  buildTime: number; // 秒
}

/** 施設の効果タイプ */
export type FacilityEffect =
  | { type: "energy_bonus"; value: number }           // カードのエナジー上限+
  | { type: "energy_percent"; value: number }         // カードのエナジー%増加
  | { type: "speed_bonus"; value: number }            // カードのスピード+
  | { type: "skill_power"; value: number }            // スキル効果%増加
  | { type: "drop_rate"; value: number }              // ドロップ率%増加
  | { type: "exp_bonus"; value: number }              // 経験値%増加
  | { type: "storage_capacity"; value: number }       // 倉庫容量+
  | { type: "unit_capacity"; value: number }          // ユニット上限+
  | { type: "passive_energy_regen"; value: number }   // 時間あたりエナジー回復
  | { type: "resource_production"; resourceId: string; value: number } // 資源生産
  | { type: "home_size"; value: number };             // 本拠地マス拡張

/** 施設の定義 */
export interface FacilityDef {
  id: FacilityId;
  name: string;
  icon: string;
  category: FacilityCategory;
  description: string;
  maxLevel: number;
  levels: FacilityLevelEffect[];
  /** 建設に必要な本拠地拡張レベル（省略時は0=拡張不要） */
  requiredExpansionLevel?: number;
}

/** 全施設定義 */
export const FACILITIES: Record<FacilityId, FacilityDef> = {
  // === 生産系施設 ===
  energy_well: {
    id: "energy_well",
    name: "エナジーの泉",
    icon: "⛲",
    category: "production",
    description: "エナジーを生成する神秘の泉",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "カードのエナジー上限+5",
        effect: { type: "energy_bonus", value: 5 },
        cost: [
          { itemId: "ancient_stone", count: 20 },
          { itemId: "rotten_wood", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "カードのエナジー上限+10",
        effect: { type: "energy_bonus", value: 10 },
        cost: [
          { itemId: "ancient_stone", count: 50 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "カードのエナジー上限+18",
        effect: { type: "energy_bonus", value: 18 },
        cost: [
          { itemId: "ancient_stone", count: 100 },
          { itemId: "mystic_crystal", count: 30 },
          { itemId: "shining_magicstone", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "カードのエナジー上限+28",
        effect: { type: "energy_bonus", value: 28 },
        cost: [
          { itemId: "ancient_stone", count: 200 },
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "カードのエナジー上限+40",
        effect: { type: "energy_bonus", value: 40 },
        cost: [
          { itemId: "ancient_stone", count: 400 },
          { itemId: "shining_magicstone", count: 50 },
          { itemId: "guardian_core", count: 5 },
          { itemId: "ancient_kings_seal", count: 1 },
        ],
        buildTime: 3600,
      },
    ],
  },

  crystal_mine: {
    id: "crystal_mine",
    name: "水晶鉱山",
    icon: "💎",
    category: "production",
    description: "神秘の水晶を採掘する施設",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "1時間ごとに神秘の水晶×1を生産",
        effect: { type: "resource_production", resourceId: "mystic_crystal", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 40 },
          { itemId: "rusty_gear", count: 20 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "1時間ごとに神秘の水晶×3を生産",
        effect: { type: "resource_production", resourceId: "mystic_crystal", value: 3 },
        cost: [
          { itemId: "ancient_stone", count: 100 },
          { itemId: "refined_iron", count: 30 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "1時間ごとに神秘の水晶×6を生産",
        effect: { type: "resource_production", resourceId: "mystic_crystal", value: 6 },
        cost: [
          { itemId: "ancient_stone", count: 200 },
          { itemId: "golden_gear", count: 3 },
          { itemId: "guardian_core", count: 1 },
        ],
        buildTime: 1800,
      },
    ],
  },

  lumber_mill: {
    id: "lumber_mill",
    name: "製材所",
    icon: "🪓",
    category: "production",
    description: "木材を加工する施設",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "1時間ごとに朽ちた木材×3を生産",
        effect: { type: "resource_production", resourceId: "rotten_wood", value: 3 },
        cost: [
          { itemId: "rotten_wood", count: 30 },
          { itemId: "rusty_gear", count: 10 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "1時間ごとに朽ちた木材×8を生産",
        effect: { type: "resource_production", resourceId: "rotten_wood", value: 8 },
        cost: [
          { itemId: "rotten_wood", count: 80 },
          { itemId: "refined_iron", count: 15 },
        ],
        buildTime: 300,
      },
      {
        level: 3,
        description: "1時間ごとに朽ちた木材×15を生産",
        effect: { type: "resource_production", resourceId: "rotten_wood", value: 15 },
        cost: [
          { itemId: "rotten_wood", count: 150 },
          { itemId: "golden_gear", count: 2 },
        ],
        buildTime: 900,
      },
    ],
  },

  // === 軍事系施設 ===
  barracks: {
    id: "barracks",
    name: "兵舎",
    icon: "🏠",
    category: "military",
    description: "ユニット編成数を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "ユニット上限+1",
        effect: { type: "unit_capacity", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rotten_wood", count: 20 },
          { itemId: "broken_brick", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "ユニット上限+2",
        effect: { type: "unit_capacity", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "reinforced_fiber", count: 15 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "ユニット上限+3",
        effect: { type: "unit_capacity", value: 3 },
        cost: [
          { itemId: "ancient_stone", count: 150 },
          { itemId: "golden_gear", count: 2 },
          { itemId: "guardian_core", count: 1 },
        ],
        buildTime: 1800,
      },
    ],
  },

  training_ground: {
    id: "training_ground",
    name: "訓練場",
    icon: "⚔️",
    category: "military",
    description: "カードの能力を強化",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "カードのエナジー+10%",
        effect: { type: "energy_percent", value: 10 },
        cost: [
          { itemId: "ancient_stone", count: 25 },
          { itemId: "reinforced_fiber", count: 10 },
        ],
        buildTime: 90,
      },
      {
        level: 2,
        description: "カードのエナジー+20%",
        effect: { type: "energy_percent", value: 20 },
        cost: [
          { itemId: "ancient_stone", count: 60 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "reinforced_fiber", count: 20 },
        ],
        buildTime: 300,
      },
      {
        level: 3,
        description: "カードのエナジー+35%",
        effect: { type: "energy_percent", value: 35 },
        cost: [
          { itemId: "ancient_stone", count: 120 },
          { itemId: "shining_magicstone", count: 10 },
          { itemId: "ancient_blueprint", count: 3 },
        ],
        buildTime: 900,
      },
      {
        level: 4,
        description: "カードのエナジー+50%",
        effect: { type: "energy_percent", value: 50 },
        cost: [
          { itemId: "ancient_stone", count: 200 },
          { itemId: "shining_magicstone", count: 25 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 2400,
      },
      {
        level: 5,
        description: "カードのエナジー+70%",
        effect: { type: "energy_percent", value: 70 },
        cost: [
          { itemId: "ancient_stone", count: 350 },
          { itemId: "shining_magicstone", count: 50 },
          { itemId: "guardian_core", count: 4 },
          { itemId: "dragon_scale", count: 1 },
        ],
        buildTime: 3600,
      },
    ],
  },

  armory: {
    id: "armory",
    name: "武器庫",
    icon: "🗡️",
    category: "military",
    description: "カードのスピードを強化",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "カードのスピード+2",
        effect: { type: "speed_bonus", value: 2 },
        cost: [
          { itemId: "refined_iron", count: 20 },
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "カードのスピード+4",
        effect: { type: "speed_bonus", value: 4 },
        cost: [
          { itemId: "refined_iron", count: 50 },
          { itemId: "golden_gear", count: 2 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "カードのスピード+7",
        effect: { type: "speed_bonus", value: 7 },
        cost: [
          { itemId: "refined_iron", count: 100 },
          { itemId: "golden_gear", count: 5 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
    ],
  },

  // === 研究系施設 ===
  research_lab: {
    id: "research_lab",
    name: "研究所",
    icon: "🔬",
    category: "research",
    description: "経験値獲得量を増加",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "経験値+20%",
        effect: { type: "exp_bonus", value: 20 },
        cost: [
          { itemId: "ancient_blueprint", count: 3 },
          { itemId: "magic_shard", count: 20 },
        ],
        buildTime: 180,
      },
      {
        level: 2,
        description: "経験値+50%",
        effect: { type: "exp_bonus", value: 50 },
        cost: [
          { itemId: "ancient_blueprint", count: 8 },
          { itemId: "mystic_crystal", count: 30 },
          { itemId: "shining_magicstone", count: 5 },
        ],
        buildTime: 900,
      },
      {
        level: 3,
        description: "経験値+100%",
        effect: { type: "exp_bonus", value: 100 },
        cost: [
          { itemId: "ancient_blueprint", count: 15 },
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 2400,
      },
    ],
  },

  magic_tower: {
    id: "magic_tower",
    name: "魔法塔",
    icon: "🗼",
    category: "research",
    description: "スキル効果を強化",
    maxLevel: 4,
    requiredExpansionLevel: 1,
    levels: [
      {
        level: 1,
        description: "スキル効果+10%",
        effect: { type: "skill_power", value: 10 },
        cost: [
          { itemId: "mystic_crystal", count: 25 },
          { itemId: "magic_shard", count: 30 },
        ],
        buildTime: 180,
      },
      {
        level: 2,
        description: "スキル効果+25%",
        effect: { type: "skill_power", value: 25 },
        cost: [
          { itemId: "mystic_crystal", count: 60 },
          { itemId: "shining_magicstone", count: 10 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "スキル効果+45%",
        effect: { type: "skill_power", value: 45 },
        cost: [
          { itemId: "shining_magicstone", count: 30 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "ancient_kings_seal", count: 1 },
        ],
        buildTime: 1800,
      },
      {
        level: 4,
        description: "スキル効果+70%",
        effect: { type: "skill_power", value: 70 },
        cost: [
          { itemId: "shining_magicstone", count: 60 },
          { itemId: "guardian_core", count: 5 },
          { itemId: "dragon_scale", count: 2 },
        ],
        buildTime: 3600,
      },
    ],
  },

  skill_shrine: {
    id: "skill_shrine",
    name: "スキルの祠",
    icon: "⛩️",
    category: "research",
    description: "新しいスキルを習得できる",
    maxLevel: 2,
    requiredExpansionLevel: 1,
    levels: [
      {
        level: 1,
        description: "スキルの書を使用可能",
        effect: { type: "skill_power", value: 0 },
        cost: [
          { itemId: "ancient_stone", count: 50 },
          { itemId: "magic_shard", count: 40 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 300,
      },
      {
        level: 2,
        description: "上位スキルの書を使用可能",
        effect: { type: "skill_power", value: 5 },
        cost: [
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "ancient_kings_seal", count: 1 },
        ],
        buildTime: 1200,
      },
    ],
  },

  // === 特殊系施設 ===
  warehouse: {
    id: "warehouse",
    name: "倉庫",
    icon: "🏪",
    category: "special",
    description: "アイテム保管量を増加",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "アイテム保管量+100",
        effect: { type: "storage_capacity", value: 100 },
        cost: [
          { itemId: "rotten_wood", count: 40 },
          { itemId: "broken_brick", count: 30 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "アイテム保管量+300",
        effect: { type: "storage_capacity", value: 300 },
        cost: [
          { itemId: "rotten_wood", count: 100 },
          { itemId: "refined_iron", count: 20 },
        ],
        buildTime: 300,
      },
      {
        level: 3,
        description: "アイテム保管量+600",
        effect: { type: "storage_capacity", value: 600 },
        cost: [
          { itemId: "rotten_wood", count: 200 },
          { itemId: "golden_gear", count: 3 },
        ],
        buildTime: 900,
      },
    ],
  },

  watchtower: {
    id: "watchtower",
    name: "見張り塔",
    icon: "🗼",
    category: "special",
    description: "遺跡の発見率を上昇",
    maxLevel: 2,
    levels: [
      {
        level: 1,
        description: "遺跡発見率+25%",
        effect: { type: "drop_rate", value: 25 },
        cost: [
          { itemId: "ancient_stone", count: 35 },
          { itemId: "rotten_wood", count: 25 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "遺跡発見率+50%",
        effect: { type: "drop_rate", value: 50 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "mystic_crystal", count: 15 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 600,
      },
    ],
  },

  altar: {
    id: "altar",
    name: "祭壇",
    icon: "🛕",
    category: "special",
    description: "ドロップ率を上昇",
    maxLevel: 3,
    requiredExpansionLevel: 2,
    levels: [
      {
        level: 1,
        description: "ドロップ率+15%",
        effect: { type: "drop_rate", value: 15 },
        cost: [
          { itemId: "mystic_crystal", count: 20 },
          { itemId: "magic_shard", count: 25 },
        ],
        buildTime: 180,
      },
      {
        level: 2,
        description: "ドロップ率+35%",
        effect: { type: "drop_rate", value: 35 },
        cost: [
          { itemId: "mystic_crystal", count: 50 },
          { itemId: "shining_magicstone", count: 8 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "ドロップ率+60%",
        effect: { type: "drop_rate", value: 60 },
        cost: [
          { itemId: "shining_magicstone", count: 25 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "dragon_scale", count: 1 },
        ],
        buildTime: 1800,
      },
    ],
  },

  home_expansion: {
    id: "home_expansion",
    name: "本拠地拡張",
    icon: "🏗️",
    category: "special",
    description: "本拠地のマス数を拡大",
    maxLevel: 4,
    levels: [
      {
        level: 1,
        description: "本拠地 9×9マス（+2）",
        effect: { type: "home_size", value: 9 },
        cost: [
          { itemId: "ancient_stone", count: 50 },
          { itemId: "rotten_wood", count: 40 },
          { itemId: "iron_ore", count: 30 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "本拠地 11×11マス（+2）",
        effect: { type: "home_size", value: 11 },
        cost: [
          { itemId: "ancient_stone", count: 100 },
          { itemId: "iron_ore", count: 60 },
          { itemId: "mystic_crystal", count: 20 },
        ],
        buildTime: 300,
      },
      {
        level: 3,
        description: "本拠地 13×13マス（+2）",
        effect: { type: "home_size", value: 13 },
        cost: [
          { itemId: "ancient_stone", count: 200 },
          { itemId: "mystic_crystal", count: 50 },
          { itemId: "shining_magicstone", count: 10 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "本拠地 15×15マス（+2）",
        effect: { type: "home_size", value: 15 },
        cost: [
          { itemId: "shining_magicstone", count: 30 },
          { itemId: "guardian_core", count: 3 },
          { itemId: "dragon_scale", count: 2 },
        ],
        buildTime: 1200,
      },
    ],
  },
};

/** 施設IDから施設定義を取得 */
export function getFacility(id: FacilityId): FacilityDef {
  return FACILITIES[id];
}

/** 建設に必要な素材が足りているか確認 */
export function canBuildFacility(
  facilityId: FacilityId,
  level: number,
  inventory: InventoryItem[],
  currentExpansionLevel: number = 0
): boolean {
  const facility = FACILITIES[facilityId];
  if (!facility || level < 1 || level > facility.maxLevel) return false;

  // 本拠地拡張レベルチェック
  const requiredLevel = facility.requiredExpansionLevel ?? 0;
  if (currentExpansionLevel < requiredLevel) return false;
  
  const levelDef = facility.levels[level - 1];
  if (!levelDef) return false;
  
  for (const cost of levelDef.cost) {
    if (getItemCount(inventory, cost.itemId) < cost.count) {
      return false;
    }
  }
  return true;
}

/** 施設が本拠地拡張レベル要件を満たしているか確認 */
export function meetsExpansionRequirement(
  facilityId: FacilityId,
  currentExpansionLevel: number
): boolean {
  const facility = FACILITIES[facilityId];
  if (!facility) return false;
  const requiredLevel = facility.requiredExpansionLevel ?? 0;
  return currentExpansionLevel >= requiredLevel;
}

/** 施設効果を集計 */
export interface FacilityBonuses {
  energyBonus: number;
  energyPercent: number;
  speedBonus: number;
  skillPower: number;
  dropRate: number;
  expBonus: number;
  storageCapacity: number;
  unitCapacity: number;
  /** 本拠地拡張レベル（0=未拡張、1=9x9、2=11x11...） */
  expansionLevel: number;
  /** 本拠地マスサイズ（7, 9, 11, 13, 15） */
  homeSize: number;
}

export function calculateFacilityBonuses(
  builtFacilities: Map<FacilityId, number>
): FacilityBonuses {
  const bonuses: FacilityBonuses = {
    energyBonus: 0,
    energyPercent: 0,
    speedBonus: 0,
    skillPower: 0,
    dropRate: 0,
    expBonus: 0,
    storageCapacity: 0,
    unitCapacity: 0,
    expansionLevel: 0,
    homeSize: 7,
  };

  for (const [facilityId, level] of builtFacilities) {
    if (level < 1) continue;
    const facility = FACILITIES[facilityId];
    if (!facility) continue;
    
    const levelDef = facility.levels[level - 1];
    if (!levelDef) continue;
    
    const effect = levelDef.effect;
    switch (effect.type) {
      case "energy_bonus":
        bonuses.energyBonus += effect.value;
        break;
      case "energy_percent":
        bonuses.energyPercent += effect.value;
        break;
      case "speed_bonus":
        bonuses.speedBonus += effect.value;
        break;
      case "skill_power":
        bonuses.skillPower += effect.value;
        break;
      case "drop_rate":
        bonuses.dropRate += effect.value;
        break;
      case "exp_bonus":
        bonuses.expBonus += effect.value;
        break;
      case "storage_capacity":
        bonuses.storageCapacity += effect.value;
        break;
      case "unit_capacity":
        bonuses.unitCapacity += effect.value;
        break;
      case "home_size":
        bonuses.homeSize = Math.max(bonuses.homeSize, effect.value);
        break;
    }

    // 本拠地拡張施設のレベルを記録
    if (facilityId === "home_expansion") {
      bonuses.expansionLevel = level;
    }
  }

  return bonuses;
}

/** 施設ボーナスを適用したエナジーを計算 */
export function applyEnergyBonus(baseEnergy: number, bonuses: FacilityBonuses): number {
  const withBonus = baseEnergy + bonuses.energyBonus;
  const withPercent = withBonus * (1 + bonuses.energyPercent / 100);
  return Math.floor(withPercent);
}

/** カテゴリ別に施設を取得（本拠地拡張は城マス専用のため除外） */
export function getFacilitiesByCategory(category: FacilityCategory): FacilityDef[] {
  return Object.values(FACILITIES).filter(
    (f) => f.category === category && f.id !== "home_expansion"
  );
}

/** 本拠地拡張施設を取得（城マス専用） */
export function getHomeExpansionFacility(): FacilityDef | undefined {
  return FACILITIES.home_expansion;
}

/** 全カテゴリを取得 */
export const FACILITY_CATEGORIES: { id: FacilityCategory; name: string; icon: string }[] = [
  { id: "production", name: "生産", icon: "⚒️" },
  { id: "military", name: "軍事", icon: "⚔️" },
  { id: "research", name: "研究", icon: "📚" },
  { id: "special", name: "特殊", icon: "✨" },
];
