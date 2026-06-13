/**
 * 戦闘ヘルパー — 攻撃/援軍バリデーション・隣接判定
 */

import type { GameState, Territory } from "../store";
import { getDeployableOwnerIds } from "../shared/game-state";
import { gameState, getLocalPlayerId } from "../store";
import {
  GRID_COLS,
  GRID_ROWS,
  getMarchFromTerritory,
  getPlayerHomeTerritoryId,
  isPlayerHomeTile,
  tryParseTerritoryId,
} from "./territories";

let cachedTerritoryAtMap: Map<string, string> | null = null;
let cachedTerritoriesRef: Territory[] | null = null;

function getTerritoryAtMap(state: GameState): Map<string, string> {
  if (cachedTerritoriesRef === state.territories && cachedTerritoryAtMap) {
    return cachedTerritoryAtMap;
  }
  const m = new Map<string, string>();
  for (const t of state.territories) {
    const p = tryParseTerritoryId(t.id);
    if (p) m.set(`${p.col},${p.row}`, t.id);
  }
  cachedTerritoryAtMap = m;
  cachedTerritoriesRef = state.territories;
  return m;
}

/** サーバー `attack_base_owner_ids` と同等 */
export function getAttackBaseOwnerIds(state: GameState, actingPlayerId: string): string[] {
  const ids = new Set<string>([actingPlayerId]);
  const player = state.players[actingPlayerId];
  if (player) {
    for (const oid of player.allied_player_ids) ids.add(oid);
  }
  for (const alliance of state.alliances ?? []) {
    if (alliance.member_ids.includes(actingPlayerId)) {
      for (const memberId of alliance.member_ids) ids.add(memberId);
    }
  }
  return [...ids];
}

/** 援軍を送れる領か（自領・クランメンバー・配下プレイヤーの領） */
export function canReceiveReinforcement(state: GameState, territory: Territory): boolean {
  if (!territory.owner_id) return false;
  return getDeployableOwnerIds(state, getLocalPlayerId()).includes(territory.owner_id);
}

/** 自陣営の領に隣接しているマスだけ攻撃可能（4方向隣接・サーバー `is_attackable_target` 相当） */
export function isAttackable(state: GameState, targetId: string): boolean {
  const pos = tryParseTerritoryId(targetId);
  if (!pos) return false;
  const { col, row } = pos;
  const territoryAt = getTerritoryAtMap(state);
  const territoryById = new Map(state.territories.map((t) => [t.id, t]));
  const baseOwners = new Set(getAttackBaseOwnerIds(state, getLocalPlayerId()));
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
    if (t?.owner_id && baseOwners.has(t.owner_id)) return true;
  }
  return false;
}

/** 攻撃遠征の出発領地（本拠が隣なら本拠、そうでなければ隣接する前線基地など自領） */
export function getAdjacentAttackSource(state: GameState, targetId: string): string | null {
  const playerId = getLocalPlayerId();
  const homeId = getPlayerHomeTerritoryId(state, playerId);
  return getMarchFromTerritory(state, playerId, targetId, homeId);
}

/** ログイン中プレイヤーの本拠地（ホーム画面へ遷移するマス） */
export function isHomeTerritory(id: string): boolean {
  const playerId = getLocalPlayerId();
  const homeId = getPlayerHomeTerritoryId(gameState, playerId);
  const territory = gameState.territories.find((t) => t.id === id);
  return isPlayerHomeTile(id, territory, playerId, homeId);
}
