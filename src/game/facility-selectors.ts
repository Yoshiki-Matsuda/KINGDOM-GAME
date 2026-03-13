import {
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

export function getHomeGridSize(state: GameState, now: number = Date.now()): number {
  return getFacilityBonusesForState(state, now).homeSize;
}

export function getExpansionLevel(state: GameState, now: number = Date.now()): number {
  return getFacilityBonusesForState(state, now).expansionLevel;
}

export function getUnitCapacity(state: GameState, now: number = Date.now()): number {
  return getFacilityBonusesForState(state, now).unitCapacity;
}

export function getInventoryForState(state: GameState): InventoryItem[] {
  return getPlayerInventory(state);
}

export function getFacilitiesForState(state: GameState): BuiltFacility[] {
  return getPlayerFacilities(state);
}
