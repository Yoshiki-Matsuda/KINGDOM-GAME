/**
 * 戦闘ヘルパー — 攻撃/援軍バリデーション・隣接判定
 */

import type { GameState, Territory } from "../store";
import { GRID_COLS, GRID_ROWS, HOME_TERRITORY_ID, parseTerritoryId } from "../map-view";

// --- Territory キャッシュ ---
let cachedTerritoryAtMap: Map<string, string> | null = null;
let cachedTerritoriesRef: Territory[] | null = null;

function getTerritoryAtMap(state: GameState): Map<string, string> {
  if (cachedTerritoriesRef === state.territories && cachedTerritoryAtMap) {
    return cachedTerritoryAtMap;
  }
  const m = new Map<string, string>();
  for (const t of state.territories) {
    const p = parseTerritoryId(t.id);
    if (p) m.set(`${p.col},${p.row}`, t.id);
  }
  cachedTerritoryAtMap = m;
  cachedTerritoriesRef = state.territories;
  return m;
}

/** 援軍を送れる領か（自領・クランメンバー・配下プレイヤーの領） */
export function canReceiveReinforcement(state: GameState, territory: Territory): boolean {
  if (!territory.owner_id) return false;
  const ids = state.deployable_owner_ids ?? [];
  return territory.owner_id === "player" || ids.includes(territory.owner_id);
}

/** 本拠地または自領に隣接しているマスだけ攻撃可能（4方向隣接） */
export function isAttackable(state: GameState, targetId: string): boolean {
  return getAdjacentAttackSource(state, targetId) !== null;
}

/** 攻撃先に隣接する自領を1つ返す（攻撃元の自動選択用） */
export function getAdjacentAttackSource(state: GameState, targetId: string): string | null {
  const pos = parseTerritoryId(targetId);
  if (!pos) return null;
  const { col, row } = pos;
  const territoryAt = getTerritoryAtMap(state);
  const territoryById = new Map(state.territories.map((t) => [t.id, t]));
  const neighbors = [
    [col - 1, row],
    [col + 1, row],
    [col, row - 1],
    [col, row + 1],
  ].filter(([c, r]) => c >= 0 && c < GRID_COLS && r >= 0 && r < GRID_ROWS);
  for (const [c, r] of neighbors) {
    const id = territoryAt.get(`${c},${r}`);
    if (!id) continue;
    const t = territoryById.get(id);
    if (t?.owner_id === "player") return id;
  }
  return null;
}

/** 本拠地は常に c_24_24 固定 */
export function isHomeTerritory(id: string): boolean {
  return id === HOME_TERRITORY_ID;
}
