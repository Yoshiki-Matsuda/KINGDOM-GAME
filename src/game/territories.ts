import { getGridCols, getGridRows } from "../store";
import { DEFAULT_WORLD_CONFIG } from "../shared/game-state";

/** 開発用フォールバック（サーバー未接続時） */
export const GRID_COLS = DEFAULT_WORLD_CONFIG.cols;
export const GRID_ROWS = DEFAULT_WORLD_CONFIG.rows;
export const HOME_COL = DEFAULT_WORLD_CONFIG.home_col;
export const HOME_ROW = DEFAULT_WORLD_CONFIG.home_row;
export const HOME_TERRITORY_ID = `c_${HOME_COL}_${HOME_ROW}`;

export function getWorldGridCols(): number {
  return getGridCols();
}

export function getWorldGridRows(): number {
  return getGridRows();
}

export interface TerritoryCoord {
  col: number;
  row: number;
}

export function tryParseTerritoryId(id: string): TerritoryCoord | null {
  if (id.startsWith("c_")) {
    const parts = id.substring(2).split("_");
    if (parts.length === 2) {
      const col = parseInt(parts[0], 10);
      const row = parseInt(parts[1], 10);
      if (Number.isFinite(col) && Number.isFinite(row)) {
        return { col, row };
      }
    }
  }

  const fallbackParts = id.split("-");
  if (fallbackParts.length === 2) {
    const col = parseInt(fallbackParts[0], 10);
    const row = parseInt(fallbackParts[1], 10);
    if (Number.isFinite(col) && Number.isFinite(row)) {
      return { col, row };
    }
  }

  return null;
}

export function parseTerritoryId(id: string): TerritoryCoord {
  return tryParseTerritoryId(id) ?? { col: 0, row: 0 };
}

export function formatTerritoryId(col: number, row: number): string {
  return `c_${col}_${row}`;
}

export function isWithinWorldGrid(col: number, row: number): boolean {
  return col >= 0 && col < getWorldGridCols() && row >= 0 && row < getWorldGridRows();
}

export function getDistanceFromHome(territoryId: string, homeTerritoryId: string = HOME_TERRITORY_ID): number {
  const home = tryParseTerritoryId(homeTerritoryId);
  const target = tryParseTerritoryId(territoryId);
  if (!home || !target) return 0;
  return Math.abs(target.col - home.col) + Math.abs(target.row - home.row);
}

export function getDistanceBetweenTerritories(fromId: string, toId: string): number {
  const from = tryParseTerritoryId(fromId);
  const to = tryParseTerritoryId(toId);
  if (!from || !to) return 0;
  return Math.abs(from.col - to.col) + Math.abs(from.row - to.row);
}

/** 遠征の出発領地（隣接する自領。本拠が隣なら本拠、そうでなければ隣接する前線基地を優先） */
export function getMarchFromTerritory(
  state: { territories: { id: string; owner_id?: string | null; is_base?: boolean }[] },
  playerId: string,
  toId: string,
  homeTerritoryId: string,
): string | null {
  if (getDistanceBetweenTerritories(homeTerritoryId, toId) === 1) {
    return homeTerritoryId;
  }
  let fallback: string | null = null;
  for (const t of state.territories) {
    if (t.owner_id === playerId && t.id !== toId) {
      if (getDistanceBetweenTerritories(t.id, toId) === 1) {
        if (t.is_base) return t.id;
        if (!fallback) fallback = t.id;
      }
    }
  }
  return fallback;
}

export function isHomeTerritoryId(id: string): boolean {
  return id === HOME_TERRITORY_ID;
}

interface HomeLookupState {
  players: Record<string, { home_territory_id: string }>;
  territories?: { id: string; owner_id?: string | null; is_base?: boolean }[];
}

export function getPlayerHomeTerritoryId(
  state: HomeLookupState & { ai_factions?: { faction_id: string; home_territory_id: string }[] },
  playerId: string,
): string {
  const fromPlayer = state.players[playerId]?.home_territory_id;
  if (fromPlayer) return fromPlayer;

  if (playerId.startsWith("ai_")) {
    const suffix = playerId.replace(/^ai_/, "");
    const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
    if (faction?.home_territory_id) return faction.home_territory_id;
  }

  const ownedBase = state.territories?.find(
    (t) => t.owner_id === playerId && t.is_base,
  );
  if (ownedBase) return ownedBase.id;
  return HOME_TERRITORY_ID;
}

export function isPlayerHomeTile(
  territoryId: string,
  territory: { owner_id?: string | null; is_base?: boolean } | undefined,
  playerId: string,
  homeTerritoryId: string,
): boolean {
  if (territoryId === homeTerritoryId) return true;
  return territory?.owner_id === playerId && territory.is_base === true;
}

/** 他プレイヤーの本拠地（占領した中立マス・前線基地と区別する） */
export function isEnemyHomeTile(
  territoryId: string,
  territory: { owner_id?: string | null; is_base?: boolean } | undefined,
  localPlayerId: string,
  state: HomeLookupState & { ai_factions?: { faction_id: string; home_territory_id: string }[] },
): boolean {
  const owner = territory?.owner_id;
  if (!owner || owner === localPlayerId || owner === "barbarian") return false;
  if (owner.startsWith("ai_")) return false;
  return territoryId === getPlayerHomeTerritoryId(state, owner);
}

/** AI 勢力の本拠地 */
export function isAiHomeTile(
  territoryId: string,
  territory: { owner_id?: string | null } | undefined,
  state: HomeLookupState & { ai_factions?: { faction_id: string; home_territory_id: string }[] },
): boolean {
  const owner = territory?.owner_id;
  if (!owner || !owner.startsWith("ai_")) return false;
  return territoryId === getPlayerHomeTerritoryId(state, owner);
}

export function isAiOwnerId(ownerId: string | null | undefined): boolean {
  return !!ownerId && ownerId.startsWith("ai_");
}
