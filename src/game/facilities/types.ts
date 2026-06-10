/**
 * 本拠地施設の型定義
 */

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
