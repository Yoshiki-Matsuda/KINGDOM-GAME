export const GRID_COLS = 48;
export const GRID_ROWS = 48;
export const HOME_COL = 24;
export const HOME_ROW = 24;
export const HOME_TERRITORY_ID = `c_${HOME_COL}_${HOME_ROW}`;

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
  return col >= 0 && col < GRID_COLS && row >= 0 && row < GRID_ROWS;
}

export function getDistanceFromHome(territoryId: string): number {
  const position = tryParseTerritoryId(territoryId);
  if (!position) return 0;
  return Math.abs(position.col - HOME_COL) + Math.abs(position.row - HOME_ROW);
}

export function getDistanceBetweenTerritories(fromId: string, toId: string): number {
  const from = tryParseTerritoryId(fromId);
  const to = tryParseTerritoryId(toId);
  if (!from || !to) return 0;
  return Math.abs(from.col - to.col) + Math.abs(from.row - to.row);
}

export function isHomeTerritoryId(id: string): boolean {
  return id === HOME_TERRITORY_ID;
}
