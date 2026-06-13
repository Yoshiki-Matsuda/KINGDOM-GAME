/**
 * ユニット編成のサーバー同期（永続化）
 */

import type { FormedUnit } from "../store";
import {
  formedUnitsList,
  gameState,
  getLocalPlayerId,
  isPlayerIdentityResolved,
  setFormedUnitsList,
  setGameState,
  setNextFormedUnitId,
  ws,
} from "../store";
import {
  getPlayerCardMonsterCounts,
  getPlayerData,
  MAX_MONSTER_COUNT_PER_CARD_SLOT,
  MIN_MONSTER_COUNT_PER_CARD_SLOT,
  setFormedUnitsAction,
  type StoredFormedUnit,
} from "../shared/game-state";
import { DEFAULT_BODY_MONSTER_COUNT } from "./characters";
import { getEffectiveCardStats } from "./effective-stats";
import { getPlayerOwnedCards } from "../shared/game-state";
import { getHomeTroops, recalcUnitStats } from "./formation";

/** WS 未接続中に commit された編成を接続後に送る */
let pendingFormedUnitsSave = false;

function toStoredFormedUnits(list: FormedUnit[]): StoredFormedUnit[] {
  return list.map((u) => ({
    id: u.id,
    name: u.name,
    indices: u.indices,
  }));
}

function buildBodyStatsForHydrate(): { counts: number[]; speeds: number[] } {
  const homeTroops = getHomeTroops();
  const serverCounts = getPlayerCardMonsterCounts(gameState, getLocalPlayerId());
  let counts =
    serverCounts.length >= homeTroops
      ? [...serverCounts]
      : [...serverCounts, ...Array(Math.max(0, homeTroops - serverCounts.length)).fill(DEFAULT_BODY_MONSTER_COUNT)];
  while (counts.length < homeTroops) counts.push(DEFAULT_BODY_MONSTER_COUNT);
  counts = counts
    .slice(0, homeTroops)
    .map((c) =>
      Math.max(MIN_MONSTER_COUNT_PER_CARD_SLOT, Math.min(MAX_MONSTER_COUNT_PER_CARD_SLOT, c)),
    );
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const speeds = Array.from({ length: homeTroops }, (_, i) => {
    const cardId = owned[i] ?? 0;
    return getEffectiveCardStats(cardId, i, gameState, getLocalPlayerId()).speed;
  });
  return { counts, speeds };
}

function syncNextFormedUnitIdFromList(units: FormedUnit[]): void {
  let max = 0;
  for (const unit of units) {
    const match = /^unit-(\d+)$/.exec(unit.id);
    if (match) max = Math.max(max, parseInt(match[1], 10));
  }
  setNextFormedUnitId(max + 1);
}

/** gameState.players 内の formed_units をローカル編成と揃える */
export function syncFormedUnitsIntoGameState(list: FormedUnit[] = formedUnitsList): void {
  if (!isPlayerIdentityResolved()) return;
  const playerId = getLocalPlayerId();
  const player = getPlayerData(gameState, playerId);
  if (!player) return;
  setGameState({
    ...gameState,
    players: {
      ...gameState.players,
      [playerId]: {
        ...player,
        formed_units: toStoredFormedUnits(list),
      },
    },
  });
}

/** サーバー state から編成を復元（gameState 更新直後に呼ぶ） */
export function hydrateFormedUnitsFromGameState(): void {
  if (!isPlayerIdentityResolved()) return;
  const stored = getPlayerData(gameState, getLocalPlayerId())?.formed_units ?? [];
  const { counts, speeds } = buildBodyStatsForHydrate();
  const units: FormedUnit[] = stored.map((u) => {
    const indices = u.indices as [number, number, number];
    const { monster_count, avgSpeed } = recalcUnitStats(indices, counts, speeds);
    return {
      id: u.id,
      name: u.name,
      indices,
      monster_count,
      avgSpeed,
    };
  });
  setFormedUnitsList(units);
  syncNextFormedUnitIdFromList(units);
}

/** 編成変更をサーバーへ送信（未接続時は pending に積む） */
export function persistFormedUnitsToServer(): void {
  if (ws?.readyState !== WebSocket.OPEN) {
    pendingFormedUnitsSave = true;
    return;
  }
  pendingFormedUnitsSave = false;
  ws.send(JSON.stringify(setFormedUnitsAction(toStoredFormedUnits(formedUnitsList))));
}

/**
 * 接続直後など、pending の編成をサーバーへ送る。
 * 送信した場合 true（呼び出し側は古い snapshot で hydrate しないこと）
 */
export function flushPendingFormedUnitsToServer(): boolean {
  if (!pendingFormedUnitsSave) return false;
  if (ws?.readyState !== WebSocket.OPEN) return false;
  pendingFormedUnitsSave = false;
  ws.send(JSON.stringify(setFormedUnitsAction(toStoredFormedUnits(formedUnitsList))));
  return true;
}

/** 編成を更新してサーバーへ保存 */
export function commitFormedUnits(list: FormedUnit[]): void {
  setFormedUnitsList(list);
  syncFormedUnitsIntoGameState(list);
  persistFormedUnitsToServer();
}
