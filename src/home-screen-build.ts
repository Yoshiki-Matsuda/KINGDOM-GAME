import type { BuiltFacility, InventoryItem } from "./shared/game-state";
import {
  FACILITIES,
  canBuildFacility,
  type FacilityId,
} from "./game/facilities";

interface TilePosition {
  col: number;
  row: number;
}

interface BuildFacilityContext {
  facilityId: FacilityId;
  level: number;
  selectedTile: TilePosition | null;
  existingFacilities: BuiltFacility[];
  inventory: InventoryItem[];
  expansionLevel: number;
  getHomeFacility: (col: number, row: number) => string | null;
  isCastleTile: (col: number, row: number) => boolean;
  devMode: boolean;
  devBuildTimeSeconds: number;
  now?: number;
}

export interface BuildFacilityResult {
  inventory: InventoryItem[];
  facilities: BuiltFacility[];
  placedFacility: { col: number; row: number; facilityId: FacilityId } | null;
}

export function createFacilityBuildState(
  context: BuildFacilityContext,
): BuildFacilityResult | null {
  const {
    facilityId,
    level,
    selectedTile,
    existingFacilities,
    inventory,
    expansionLevel,
    getHomeFacility,
    isCastleTile,
    devMode,
    devBuildTimeSeconds,
  } = context;

  const isExpansion = facilityId === "home_expansion";

  if (!isExpansion && level === 1 && !selectedTile) return null;
  if (isExpansion && (!selectedTile || !isCastleTile(selectedTile.col, selectedTile.row))) return null;
  if (level === 1 && existingFacilities.some((facility) => facility.facility_id === facilityId)) return null;

  const facility = FACILITIES[facilityId];
  if (!facility) return null;

  const levelDef = facility.levels[level - 1];
  if (!levelDef) return null;
  if (!canBuildFacility(facilityId, level, inventory, expansionLevel)) return null;

  const nextInventory = [...inventory];
  for (const cost of levelDef.cost) {
    const index = nextInventory.findIndex((item) => item.item_id === cost.itemId);
    if (index < 0) continue;

    nextInventory[index] = {
      ...nextInventory[index],
      count: nextInventory[index].count - cost.count,
    };
    if (nextInventory[index].count <= 0) {
      nextInventory.splice(index, 1);
    }
  }

  const nextFacilities = [...existingFacilities];
  const existingIndex = nextFacilities.findIndex((builtFacility) => {
    if (builtFacility.facility_id !== facilityId) return false;
    if (isExpansion) return true;
    if (!selectedTile) return true;
    if (builtFacility.position) {
      return (
        builtFacility.position.col === selectedTile.col &&
        builtFacility.position.row === selectedTile.row
      );
    }
    return getHomeFacility(selectedTile.col, selectedTile.row) === facilityId;
  });

  const buildTimeSeconds = devMode ? devBuildTimeSeconds : levelDef.buildTime;
  const buildCompleteAt = (context.now ?? Date.now()) + buildTimeSeconds * 1000;

  if (existingIndex >= 0) {
    nextFacilities[existingIndex] = {
      ...nextFacilities[existingIndex],
      level,
      build_complete_at: buildCompleteAt,
    };

    return {
      inventory: nextInventory,
      facilities: nextFacilities,
      placedFacility: null,
    };
  }

  const position = !isExpansion && selectedTile
    ? { col: selectedTile.col, row: selectedTile.row }
    : undefined;
  nextFacilities.push({
    facility_id: facilityId,
    level,
    build_complete_at: buildCompleteAt,
    position,
  });

  return {
    inventory: nextInventory,
    facilities: nextFacilities,
    placedFacility: !isExpansion && selectedTile
      ? { col: selectedTile.col, row: selectedTile.row, facilityId }
      : null,
  };
}
