/**
 * 本拠地施設の定義
 */

import type { InventoryItem } from "../shared/game-state";
import { getItemCount } from "./items";

/** 施設のカテゴリ */
export type FacilityCategory = "resource" | "military" | "race_lab" | "special";

/** 施設ID */
export type FacilityId =
  // 資源生産
  | "field"
  | "lumber_mill"
  | "ironworks"
  | "quarry"
  | "warehouse"
  | "trading_post"
  // 軍事
  | "fortress"
  | "stronghold"
  | "training_tower"
  | "monster_barracks"
  | "battle_lab"
  // 種族研究所
  | "beast_lab"
  | "demihuman_lab"
  | "spirit_lab"
  | "undead_lab"
  | "giant_lab"
  | "demon_lab"
  | "dragon_lab"
  // 特殊
  | "library"
  | "hero_statue"
  | "guardian_shrine"
  | "war_god_shrine";

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
  | { type: "monster_bonus"; value: number }
  | { type: "monster_percent"; value: number }
  | { type: "speed_bonus"; value: number }
  | { type: "skill_power"; value: number }
  | { type: "drop_rate"; value: number }
  | { type: "exp_bonus"; value: number }
  | { type: "storage_capacity"; value: number }
  | { type: "unit_capacity"; value: number }
  | { type: "resource_production"; resourceId: string; value: number }
  | { type: "market_fee_reduction"; value: number }
  | { type: "defense_bonus"; value: number }
  | { type: "attack_bonus"; value: number }
  | { type: "race_capacity"; race: string; value: number };

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
  // === 資源生産施設 ===
  field: {
    id: "field",
    name: "農場",
    icon: "🌾",
    category: "resource",
    description: "食料を生産する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "食料生産+5/h",
        effect: { type: "resource_production", resourceId: "food", value: 5 },
        cost: [
          { itemId: "rotten_wood", count: 20 },
          { itemId: "ancient_stone", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "食料生産+12/h",
        effect: { type: "resource_production", resourceId: "food", value: 12 },
        cost: [
          { itemId: "rotten_wood", count: 50 },
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rusty_gear", count: 10 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "食料生産+25/h",
        effect: { type: "resource_production", resourceId: "food", value: 25 },
        cost: [
          { itemId: "rotten_wood", count: 100 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "食料生産+45/h",
        effect: { type: "resource_production", resourceId: "food", value: 45 },
        cost: [
          { itemId: "rotten_wood", count: 200 },
          { itemId: "refined_iron", count: 50 },
          { itemId: "shining_magicstone", count: 10 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "食料生産+80/h",
        effect: { type: "resource_production", resourceId: "food", value: 80 },
        cost: [
          { itemId: "rotten_wood", count: 400 },
          { itemId: "shining_magicstone", count: 30 },
          { itemId: "guardian_core", count: 3 },
        ],
        buildTime: 3600,
      },
    ],
  },

  lumber_mill: {
    id: "lumber_mill",
    name: "製材所",
    icon: "🪓",
    category: "resource",
    description: "木材を生産する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "木材生産+5/h",
        effect: { type: "resource_production", resourceId: "wood", value: 5 },
        cost: [
          { itemId: "rotten_wood", count: 25 },
          { itemId: "rusty_gear", count: 10 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "木材生産+12/h",
        effect: { type: "resource_production", resourceId: "wood", value: 12 },
        cost: [
          { itemId: "rotten_wood", count: 60 },
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rusty_gear", count: 20 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "木材生産+25/h",
        effect: { type: "resource_production", resourceId: "wood", value: 25 },
        cost: [
          { itemId: "rotten_wood", count: 120 },
          { itemId: "refined_iron", count: 25 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "木材生産+45/h",
        effect: { type: "resource_production", resourceId: "wood", value: 45 },
        cost: [
          { itemId: "rotten_wood", count: 250 },
          { itemId: "refined_iron", count: 60 },
          { itemId: "shining_magicstone", count: 12 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "木材生産+80/h",
        effect: { type: "resource_production", resourceId: "wood", value: 80 },
        cost: [
          { itemId: "rotten_wood", count: 500 },
          { itemId: "shining_magicstone", count: 35 },
          { itemId: "guardian_core", count: 3 },
        ],
        buildTime: 3600,
      },
    ],
  },

  ironworks: {
    id: "ironworks",
    name: "鉄工所",
    icon: "⚒️",
    category: "resource",
    description: "鉄を生産する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "鉄生産+3/h",
        effect: { type: "resource_production", resourceId: "iron", value: 3 },
        cost: [
          { itemId: "ancient_stone", count: 25 },
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "鉄生産+8/h",
        effect: { type: "resource_production", resourceId: "iron", value: 8 },
        cost: [
          { itemId: "ancient_stone", count: 60 },
          { itemId: "rusty_gear", count: 30 },
          { itemId: "refined_iron", count: 15 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "鉄生産+18/h",
        effect: { type: "resource_production", resourceId: "iron", value: 18 },
        cost: [
          { itemId: "ancient_stone", count: 120 },
          { itemId: "refined_iron", count: 40 },
          { itemId: "mystic_crystal", count: 15 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "鉄生産+35/h",
        effect: { type: "resource_production", resourceId: "iron", value: 35 },
        cost: [
          { itemId: "ancient_stone", count: 250 },
          { itemId: "refined_iron", count: 80 },
          { itemId: "shining_magicstone", count: 15 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "鉄生産+60/h",
        effect: { type: "resource_production", resourceId: "iron", value: 60 },
        cost: [
          { itemId: "ancient_stone", count: 500 },
          { itemId: "shining_magicstone", count: 40 },
          { itemId: "guardian_core", count: 4 },
        ],
        buildTime: 3600,
      },
    ],
  },

  quarry: {
    id: "quarry",
    name: "石切場",
    icon: "⛏️",
    category: "resource",
    description: "石材を生産する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "石材生産+3/h",
        effect: { type: "resource_production", resourceId: "stone", value: 3 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rotten_wood", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "石材生産+8/h",
        effect: { type: "resource_production", resourceId: "stone", value: 8 },
        cost: [
          { itemId: "ancient_stone", count: 70 },
          { itemId: "rotten_wood", count: 30 },
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "石材生産+18/h",
        effect: { type: "resource_production", resourceId: "stone", value: 18 },
        cost: [
          { itemId: "ancient_stone", count: 150 },
          { itemId: "refined_iron", count: 30 },
          { itemId: "mystic_crystal", count: 12 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "石材生産+35/h",
        effect: { type: "resource_production", resourceId: "stone", value: 35 },
        cost: [
          { itemId: "ancient_stone", count: 300 },
          { itemId: "refined_iron", count: 70 },
          { itemId: "shining_magicstone", count: 12 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "石材生産+60/h",
        effect: { type: "resource_production", resourceId: "stone", value: 60 },
        cost: [
          { itemId: "ancient_stone", count: 600 },
          { itemId: "shining_magicstone", count: 35 },
          { itemId: "guardian_core", count: 3 },
          { itemId: "ancient_kings_seal", count: 1 },
        ],
        buildTime: 3600,
      },
    ],
  },

  warehouse: {
    id: "warehouse",
    name: "倉庫",
    icon: "🏚️",
    category: "resource",
    description: "資源の保管量を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "資源保管量+100",
        effect: { type: "storage_capacity", value: 100 },
        cost: [
          { itemId: "rotten_wood", count: 30 },
          { itemId: "ancient_stone", count: 20 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "資源保管量+300",
        effect: { type: "storage_capacity", value: 300 },
        cost: [
          { itemId: "rotten_wood", count: 80 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 300,
      },
      {
        level: 3,
        description: "資源保管量+600",
        effect: { type: "storage_capacity", value: 600 },
        cost: [
          { itemId: "rotten_wood", count: 200 },
          { itemId: "golden_gear", count: 3 },
          { itemId: "shining_magicstone", count: 10 },
        ],
        buildTime: 900,
      },
    ],
  },

  trading_post: {
    id: "trading_post",
    name: "交易所",
    icon: "⚖️",
    category: "resource",
    description: "取引手数料を軽減する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "手数料-2%",
        effect: { type: "market_fee_reduction", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rotten_wood", count: 20 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "手数料-4%",
        effect: { type: "market_fee_reduction", value: 4 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 15 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "手数料-6%",
        effect: { type: "market_fee_reduction", value: 6 },
        cost: [
          { itemId: "ancient_stone", count: 150 },
          { itemId: "golden_gear", count: 2 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "手数料-8%",
        effect: { type: "market_fee_reduction", value: 8 },
        cost: [
          { itemId: "shining_magicstone", count: 15 },
          { itemId: "guardian_core", count: 1 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "手数料-10%",
        effect: { type: "market_fee_reduction", value: 10 },
        cost: [
          { itemId: "shining_magicstone", count: 30 },
          { itemId: "guardian_core", count: 3 },
          { itemId: "dragon_scale", count: 1 },
        ],
        buildTime: 3600,
      },
    ],
  },

  // === 軍事施設 ===
  fortress: {
    id: "fortress",
    name: "要塞",
    icon: "🏰",
    category: "military",
    description: "防御力を強化する",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "防御力+5%",
        effect: { type: "defense_bonus", value: 5 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rotten_wood", count: 20 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "防御力+10%",
        effect: { type: "defense_bonus", value: 10 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "防御力+15%",
        effect: { type: "defense_bonus", value: 15 },
        cost: [
          { itemId: "ancient_stone", count: 150 },
          { itemId: "refined_iron", count: 50 },
          { itemId: "mystic_crystal", count: 20 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "防御力+20%",
        effect: { type: "defense_bonus", value: 20 },
        cost: [
          { itemId: "ancient_stone", count: 300 },
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 3 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "防御力+30%",
        effect: { type: "defense_bonus", value: 30 },
        cost: [
          { itemId: "ancient_stone", count: 500 },
          { itemId: "shining_magicstone", count: 50 },
          { itemId: "guardian_core", count: 5 },
          { itemId: "dragon_scale", count: 2 },
        ],
        buildTime: 3600,
      },
    ],
  },

  stronghold: {
    id: "stronghold",
    name: "砦",
    icon: "🛡️",
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
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "ユニット上限+2",
        effect: { type: "unit_capacity", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 25 },
          { itemId: "golden_gear", count: 2 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "ユニット上限+3",
        effect: { type: "unit_capacity", value: 3 },
        cost: [
          { itemId: "ancient_stone", count: 150 },
          { itemId: "golden_gear", count: 5 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
    ],
  },

  training_tower: {
    id: "training_tower",
    name: "魔獣訓練塔",
    icon: "🗼",
    category: "military",
    description: "魔獣数を%増加させる",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "魔獣数+10%",
        effect: { type: "monster_percent", value: 10 },
        cost: [
          { itemId: "ancient_stone", count: 25 },
          { itemId: "rotten_wood", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "魔獣数+20%",
        effect: { type: "monster_percent", value: 20 },
        cost: [
          { itemId: "ancient_stone", count: 60 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "魔獣数+35%",
        effect: { type: "monster_percent", value: 35 },
        cost: [
          { itemId: "ancient_stone", count: 120 },
          { itemId: "shining_magicstone", count: 10 },
          { itemId: "ancient_blueprint", count: 3 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "魔獣数+50%",
        effect: { type: "monster_percent", value: 50 },
        cost: [
          { itemId: "ancient_stone", count: 250 },
          { itemId: "shining_magicstone", count: 25 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "魔獣数+70%",
        effect: { type: "monster_percent", value: 70 },
        cost: [
          { itemId: "ancient_stone", count: 400 },
          { itemId: "shining_magicstone", count: 50 },
          { itemId: "guardian_core", count: 4 },
          { itemId: "dragon_scale", count: 1 },
        ],
        buildTime: 3600,
      },
    ],
  },

  monster_barracks: {
    id: "monster_barracks",
    name: "魔獣兵舎",
    icon: "⚔️",
    category: "military",
    description: "魔獣数上限を増やす",
    maxLevel: 5,
    levels: [
      {
        level: 1,
        description: "魔獣数上限+5",
        effect: { type: "monster_bonus", value: 5 },
        cost: [
          { itemId: "ancient_stone", count: 20 },
          { itemId: "rotten_wood", count: 15 },
        ],
        buildTime: 60,
      },
      {
        level: 2,
        description: "魔獣数上限+10",
        effect: { type: "monster_bonus", value: 10 },
        cost: [
          { itemId: "ancient_stone", count: 50 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 180,
      },
      {
        level: 3,
        description: "魔獣数上限+18",
        effect: { type: "monster_bonus", value: 18 },
        cost: [
          { itemId: "ancient_stone", count: 100 },
          { itemId: "mystic_crystal", count: 30 },
          { itemId: "shining_magicstone", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 4,
        description: "魔獣数上限+28",
        effect: { type: "monster_bonus", value: 28 },
        cost: [
          { itemId: "ancient_stone", count: 200 },
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
      {
        level: 5,
        description: "魔獣数上限+40",
        effect: { type: "monster_bonus", value: 40 },
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

  battle_lab: {
    id: "battle_lab",
    name: "戦闘研究所",
    icon: "🔬",
    category: "military",
    description: "スキル効果を強化する",
    maxLevel: 4,
    levels: [
      {
        level: 1,
        description: "スキル効果+10%",
        effect: { type: "skill_power", value: 10 },
        cost: [
          { itemId: "mystic_crystal", count: 25 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "スキル効果+25%",
        effect: { type: "skill_power", value: 25 },
        cost: [
          { itemId: "mystic_crystal", count: 60 },
          { itemId: "shining_magicstone", count: 10 },
          { itemId: "ancient_blueprint", count: 5 },
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

  // === 種族研究所 ===
  beast_lab: {
    id: "beast_lab",
    name: "獣族研究所",
    icon: "🐺",
    category: "race_lab",
    description: "獣族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "獣族枠+1",
        effect: { type: "race_capacity", race: "beast", value: 1 },
        cost: [
          { itemId: "rotten_wood", count: 30 },
          { itemId: "ancient_stone", count: 20 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "獣族枠+2",
        effect: { type: "race_capacity", race: "beast", value: 2 },
        cost: [
          { itemId: "rotten_wood", count: 80 },
          { itemId: "mystic_crystal", count: 15 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "獣族枠+3",
        effect: { type: "race_capacity", race: "beast", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 15 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  demihuman_lab: {
    id: "demihuman_lab",
    name: "亜人族研究所",
    icon: "👹",
    category: "race_lab",
    description: "亜人族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "亜人族枠+1",
        effect: { type: "race_capacity", race: "demihuman", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "rusty_gear", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "亜人族枠+2",
        effect: { type: "race_capacity", race: "demihuman", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 20 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "亜人族枠+3",
        effect: { type: "race_capacity", race: "demihuman", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 15 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  spirit_lab: {
    id: "spirit_lab",
    name: "精霊族研究所",
    icon: "✨",
    category: "race_lab",
    description: "精霊族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "精霊族枠+1",
        effect: { type: "race_capacity", race: "spirit", value: 1 },
        cost: [
          { itemId: "mystic_crystal", count: 25 },
          { itemId: "rotten_wood", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "精霊族枠+2",
        effect: { type: "race_capacity", race: "spirit", value: 2 },
        cost: [
          { itemId: "mystic_crystal", count: 60 },
          { itemId: "shining_magicstone", count: 8 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "精霊族枠+3",
        effect: { type: "race_capacity", race: "spirit", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  undead_lab: {
    id: "undead_lab",
    name: "不死族研究所",
    icon: "💀",
    category: "race_lab",
    description: "不死族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "不死族枠+1",
        effect: { type: "race_capacity", race: "undead", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 25 },
          { itemId: "mystic_crystal", count: 10 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "不死族枠+2",
        effect: { type: "race_capacity", race: "undead", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 70 },
          { itemId: "mystic_crystal", count: 25 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "不死族枠+3",
        effect: { type: "race_capacity", race: "undead", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 15 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  giant_lab: {
    id: "giant_lab",
    name: "巨人族研究所",
    icon: "⛰️",
    category: "race_lab",
    description: "巨人族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "巨人族枠+1",
        effect: { type: "race_capacity", race: "giant", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "refined_iron", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "巨人族枠+2",
        effect: { type: "race_capacity", race: "giant", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 80 },
          { itemId: "refined_iron", count: 30 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "巨人族枠+3",
        effect: { type: "race_capacity", race: "giant", value: 3 },
        cost: [
          { itemId: "refined_iron", count: 50 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  demon_lab: {
    id: "demon_lab",
    name: "魔族研究所",
    icon: "😈",
    category: "race_lab",
    description: "魔族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "魔族枠+1",
        effect: { type: "race_capacity", race: "demon", value: 1 },
        cost: [
          { itemId: "mystic_crystal", count: 20 },
          { itemId: "ancient_stone", count: 20 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "魔族枠+2",
        effect: { type: "race_capacity", race: "demon", value: 2 },
        cost: [
          { itemId: "mystic_crystal", count: 50 },
          { itemId: "shining_magicstone", count: 10 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "魔族枠+3",
        effect: { type: "race_capacity", race: "demon", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 20 },
          { itemId: "guardian_core", count: 2 },
          { itemId: "enchanted_cloth", count: 10 },
        ],
        buildTime: 1800,
      },
    ],
  },

  dragon_lab: {
    id: "dragon_lab",
    name: "龍族研究所",
    icon: "🐉",
    category: "race_lab",
    description: "龍族の編成上限を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "龍族枠+1",
        effect: { type: "race_capacity", race: "dragon", value: 1 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "mystic_crystal", count: 15 },
          { itemId: "dragon_scale", count: 1 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "龍族枠+2",
        effect: { type: "race_capacity", race: "dragon", value: 2 },
        cost: [
          { itemId: "shining_magicstone", count: 15 },
          { itemId: "dragon_scale", count: 3 },
          { itemId: "enchanted_cloth", count: 5 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "龍族枠+3",
        effect: { type: "race_capacity", race: "dragon", value: 3 },
        cost: [
          { itemId: "shining_magicstone", count: 30 },
          { itemId: "dragon_scale", count: 5 },
          { itemId: "guardian_core", count: 3 },
        ],
        buildTime: 1800,
      },
    ],
  },

  // === 特殊施設 ===
  library: {
    id: "library",
    name: "図書館",
    icon: "📚",
    category: "special",
    description: "経験値を増やす",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "経験値+20%",
        effect: { type: "exp_bonus", value: 20 },
        cost: [
          { itemId: "ancient_blueprint", count: 3 },
          { itemId: "rotten_wood", count: 25 },
        ],
        buildTime: 120,
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
        buildTime: 600,
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
        buildTime: 1800,
      },
    ],
  },

  hero_statue: {
    id: "hero_statue",
    name: "英雄像",
    icon: "🗿",
    category: "special",
    description: "スピードを強化する",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "スピード+2",
        effect: { type: "speed_bonus", value: 2 },
        cost: [
          { itemId: "ancient_stone", count: 30 },
          { itemId: "refined_iron", count: 15 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "スピード+4",
        effect: { type: "speed_bonus", value: 4 },
        cost: [
          { itemId: "refined_iron", count: 40 },
          { itemId: "golden_gear", count: 2 },
          { itemId: "ancient_blueprint", count: 2 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "スピード+7",
        effect: { type: "speed_bonus", value: 7 },
        cost: [
          { itemId: "refined_iron", count: 80 },
          { itemId: "golden_gear", count: 5 },
          { itemId: "guardian_core", count: 2 },
        ],
        buildTime: 1800,
      },
    ],
  },

  guardian_shrine: {
    id: "guardian_shrine",
    name: "守護の祠",
    icon: "⛩️",
    category: "special",
    description: "ドロップ率を上げる",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "ドロップ率+15%",
        effect: { type: "drop_rate", value: 15 },
        cost: [
          { itemId: "mystic_crystal", count: 20 },
          { itemId: "ancient_stone", count: 25 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "ドロップ率+35%",
        effect: { type: "drop_rate", value: 35 },
        cost: [
          { itemId: "mystic_crystal", count: 50 },
          { itemId: "shining_magicstone", count: 8 },
          { itemId: "enchanted_cloth", count: 5 },
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

  war_god_shrine: {
    id: "war_god_shrine",
    name: "軍神の祠",
    icon: "🗡️",
    category: "special",
    description: "攻撃力を強化する",
    maxLevel: 3,
    levels: [
      {
        level: 1,
        description: "攻撃力+5%",
        effect: { type: "attack_bonus", value: 5 },
        cost: [
          { itemId: "refined_iron", count: 25 },
          { itemId: "ancient_stone", count: 20 },
        ],
        buildTime: 120,
      },
      {
        level: 2,
        description: "攻撃力+10%",
        effect: { type: "attack_bonus", value: 10 },
        cost: [
          { itemId: "refined_iron", count: 60 },
          { itemId: "golden_gear", count: 3 },
          { itemId: "ancient_blueprint", count: 3 },
        ],
        buildTime: 600,
      },
      {
        level: 3,
        description: "攻撃力+20%",
        effect: { type: "attack_bonus", value: 20 },
        cost: [
          { itemId: "golden_gear", count: 5 },
          { itemId: "guardian_core", count: 3 },
          { itemId: "ancient_kings_seal", count: 1 },
        ],
        buildTime: 1800,
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
  monsterBonus: number;
  monsterPercent: number;
  speedBonus: number;
  skillPower: number;
  dropRate: number;
  expBonus: number;
  storageCapacity: number;
  unitCapacity: number;
  marketFeeReduction: number;
  defenseBonus: number;
  attackBonus: number;
  /** 図書館／研究施設（サーバー `facilities.rs` と同値） */
  unitCostCapBonus: number;
}

export function calculateFacilityBonuses(
  builtFacilities: Map<FacilityId, number>
): FacilityBonuses {
  const bonuses: FacilityBonuses = {
    monsterBonus: 0,
    monsterPercent: 0,
    speedBonus: 0,
    skillPower: 0,
    dropRate: 0,
    expBonus: 0,
    storageCapacity: 0,
    unitCapacity: 0,
    marketFeeReduction: 0,
    defenseBonus: 0,
    attackBonus: 0,
    unitCostCapBonus: 0,
  };

  for (const [facilityId, level] of builtFacilities) {
    if (level < 1) continue;
    const fid = facilityId as string;
    if (fid === "library" || fid === "research_lab") {
      const capBumps = [0.15, 0.35, 0.6];
      const bump = capBumps[level - 1];
      if (bump !== undefined) bonuses.unitCostCapBonus += bump;
    }
    const facility = FACILITIES[facilityId];
    if (!facility) continue;

    const levelDef = facility.levels[level - 1];
    if (!levelDef) continue;

    const effect = levelDef.effect;
    switch (effect.type) {
      case "monster_bonus":
        bonuses.monsterBonus += effect.value;
        break;
      case "monster_percent":
        bonuses.monsterPercent += effect.value;
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
      case "market_fee_reduction":
        bonuses.marketFeeReduction += effect.value;
        break;
      case "defense_bonus":
        bonuses.defenseBonus += effect.value;
        break;
      case "attack_bonus":
        bonuses.attackBonus += effect.value;
        break;
    }
  }

  return bonuses;
}

/** 施設ボーナスを適用した魔獣数を計算 */
export function applyMonsterBonus(baseMonsterCount: number, bonuses: FacilityBonuses): number {
  const withBonus = baseMonsterCount + bonuses.monsterBonus;
  const withPercent = withBonus * (1 + bonuses.monsterPercent / 100);
  return Math.floor(withPercent);
}

/** カテゴリ別に施設を取得 */
export function getFacilitiesByCategory(category: FacilityCategory): FacilityDef[] {
  return Object.values(FACILITIES).filter((f) => f.category === category);
}

/** 全カテゴリを取得 */
export const FACILITY_CATEGORIES: { id: FacilityCategory; name: string; icon: string }[] = [
  { id: "resource", name: "資源", icon: "🌾" },
  { id: "military", name: "軍事", icon: "⚔️" },
  { id: "race_lab", name: "種族研究", icon: "🔬" },
  { id: "special", name: "特殊", icon: "✨" },
];
