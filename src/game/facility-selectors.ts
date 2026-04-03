import {
  DEFAULT_PLAYER_ID,
  getPlayerData,
  getPlayerFacilities,
  getPlayerInventory,
  type BuiltFacility,
  type GameState,
  type InventoryItem,
} from "../shared/game-state";
import {
  calculateFacilityBonuses,
  type FacilityBonuses,
  type FacilityId,
} from "./facilities";

export function getCompletedFacilitiesMap(
  facilities: BuiltFacility[],
  now: number = Date.now(),
): Map<FacilityId, number> {
  const builtFacilities = new Map<FacilityId, number>();
  for (const facility of facilities) {
    if (!facility.build_complete_at || facility.build_complete_at <= now) {
      builtFacilities.set(facility.facility_id as FacilityId, facility.level);
    }
  }
  return builtFacilities;
}

export function getFacilityBonusesForState(
  state: GameState,
  now: number = Date.now(),
): FacilityBonuses {
  return calculateFacilityBonuses(getCompletedFacilitiesMap(getPlayerFacilities(state), now));
}

export function getHomeGridSize(_state: GameState, _now: number = Date.now()): number {
  return 7;
}

export function getExpansionLevel(_state: GameState, _now: number = Date.now()): number {
  return 0;
}

export function getUnitCapacity(state: GameState, now: number = Date.now()): number {
  return getFacilityBonusesForState(state, now).unitCapacity;
}

/** プレイヤー基礎＋施設ボーナス（サーバー `model_actions` の cost_cap と一致） */
const DEFAULT_UNIT_COST_CAP = 4;

export function getEffectiveUnitCostCap(state: GameState, now: number = Date.now()): number {
  const p = getPlayerData(state, DEFAULT_PLAYER_ID);
  const base = p?.unit_cost_cap ?? state.unit_cost_cap ?? DEFAULT_UNIT_COST_CAP;
  return base + getFacilityBonusesForState(state, now).unitCostCapBonus;
}

export function getInventoryForState(state: GameState): InventoryItem[] {
  return getPlayerInventory(state);
}

export function getFacilitiesForState(state: GameState): BuiltFacility[] {
  return getPlayerFacilities(state);
}
