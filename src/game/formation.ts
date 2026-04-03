/**
 * 編成ロジック — ユニットの検証・作成・解体
 */

import {
  bodyMonsterCounts, bodySpeeds,
  formedUnitsList, setFormedUnitsList,
  getNextFormedUnitId,
  setBodyMonsterCounts, setBodySpeeds,
  gameState,
} from "../store";
import { getPlayerOwnedCards } from "../shared/game-state";
import { BODIES_PER_UNIT, DEFAULT_BODY_MONSTER_COUNT, DEFAULT_BODY_SPEED, getCharacterStats } from "./characters";
import { getEffectiveUnitCostCap } from "./facility-selectors";
import { HOME_TERRITORY_ID } from "../map-view";

/** 本拠地の体数を取得 */
export function getHomeTroops(): number {
  const t = gameState.territories.find((x) => x.id === HOME_TERRITORY_ID);
  return t?.troops ?? 0;
}

/**
 * 本拠地の体数が変わったら bodyMonsterCounts/bodySpeeds を伸縮し、
 * 編成済みユニットを検証する
 */
export function validateFormedUnits(): void {
  const homeTroops = getHomeTroops();
  const counts = [...bodyMonsterCounts];
  while (counts.length < homeTroops) {
    counts.push(DEFAULT_BODY_MONSTER_COUNT);
  }
  setBodyMonsterCounts(counts.slice(0, homeTroops));

  const speeds = [...bodySpeeds];
  while (speeds.length < homeTroops) {
    speeds.push(Math.floor(Math.random() * 5) + 3);
  }
  setBodySpeeds(speeds.slice(0, homeTroops));

  // 有効なユニット: 各indexが-1（空き）または 0..homeTroops-1
  let units = formedUnitsList.filter((u) =>
    u.indices.every((i) => i === -1 || (i >= 0 && i < homeTroops))
  );
  const completeUnits = units.filter((u) => u.indices.every((i) => i >= 0));
  const incompleteUnits = units.filter((u) => u.indices.some((i) => i < 0));
  // 完成ユニットが多すぎる場合のみ削除（未完成は編集用に残す）
  let toKeep = completeUnits;
  while (toKeep.length * BODIES_PER_UNIT > homeTroops && toKeep.length > 0) {
    toKeep = toKeep.slice(0, -1);
  }
  units = [...toKeep, ...incompleteUnits];
  setFormedUnitsList(units);
}

/** ユニットの魔獣数合計/avgSpeedをindicesから再計算（-1はスキップ） */
export function recalcUnitStats(
  indices: [number, number, number],
  monsterCounts: number[],
  speeds: number[]
): { monster_count: number; avgSpeed: number } {
  const valid = indices.filter((i) => i >= 0);
  if (valid.length === 0) return { monster_count: 0, avgSpeed: 0 };
  const monster_count = valid.reduce((s, i) => s + (monsterCounts[i] ?? DEFAULT_BODY_MONSTER_COUNT), 0);
  const avgSpeed = valid.reduce((s, i) => s + (speeds[i] ?? DEFAULT_BODY_SPEED), 0) / valid.length;
  return { monster_count, avgSpeed };
}

/** 所持カードIDが本拠体インデックスに収まるものを優先し、足りなければ 0..から埋める */
function pickDefaultIndicesForOwned(): [number, number, number] | null {
  const troops = getHomeTroops();
  if (troops < BODIES_PER_UNIT) return null;
  const owned = getPlayerOwnedCards(gameState);
  const picks: number[] = [];
  const seen = new Set<number>();
  for (const cardId of owned) {
    if (cardId >= 0 && cardId < troops && !seen.has(cardId)) {
      seen.add(cardId);
      picks.push(cardId);
      if (picks.length >= 3) break;
    }
  }
  for (let b = 0; b < troops && picks.length < 3; b++) {
    if (!seen.has(b)) {
      seen.add(b);
      picks.push(b);
    }
  }
  if (picks.length < 3) return null;
  return [picks[0], picks[1], picks[2]];
}

/** 開発用: 編成が0のとき1ユニットを自動追加（所持と体インデックスのずれを抑える） */
export function ensureDevUnit(): void {
  validateFormedUnits();
  if (formedUnitsList.length !== 0 || getHomeTroops() < BODIES_PER_UNIT) return;

  const tri = pickDefaultIndicesForOwned();
  if (!tri) return;

  const cap = getEffectiveUnitCostCap(gameState);
  const devCost = tri.reduce((s, i) => s + getCharacterStats(i).cost, 0);
  if (devCost > cap + 0.0001) return;

  const [i0, i1, i2] = tri;
  const e0 = bodyMonsterCounts[i0] ?? DEFAULT_BODY_MONSTER_COUNT;
  const e1 = bodyMonsterCounts[i1] ?? DEFAULT_BODY_MONSTER_COUNT;
  const e2 = bodyMonsterCounts[i2] ?? DEFAULT_BODY_MONSTER_COUNT;
  const s0 = bodySpeeds[i0] ?? DEFAULT_BODY_SPEED;
  const s1 = bodySpeeds[i1] ?? DEFAULT_BODY_SPEED;
  const s2 = bodySpeeds[i2] ?? DEFAULT_BODY_SPEED;
  setFormedUnitsList([{
    id: `unit-${getNextFormedUnitId()}`,
    name: "ユニット1",
    indices: tri,
    monster_count: e0 + e1 + e2,
    avgSpeed: (s0 + s1 + s2) / 3,
  }]);
}
