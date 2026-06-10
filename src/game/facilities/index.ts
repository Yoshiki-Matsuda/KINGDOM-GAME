/**
 * 本拠地施設の定義（型・データ再エクスポートとロジック）
 */

import type { InventoryItem } from "../../shared/game-state";
import { getItemCount } from "../items";
import type { FacilityCategory, FacilityDef, FacilityId } from "./types";
import { FACILITIES } from "./data";

export * from "./types";
export { FACILITIES } from "./data";

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
