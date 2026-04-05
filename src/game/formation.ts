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
import {
  getPlayerCardMonsterCounts,
  getPlayerOwnedCards,
  MAX_MONSTER_COUNT_PER_CARD_SLOT,
  MIN_MONSTER_COUNT_PER_CARD_SLOT,
} from "../shared/game-state";
import { BODIES_PER_UNIT, DEFAULT_BODY_MONSTER_COUNT, DEFAULT_BODY_SPEED, getCharacterStats } from "./characters";
import { getEffectiveUnitCostCap } from "./facility-selectors";
import { HOME_TERRITORY_ID } from "../map-view";

/** KC: 編成3枠のうちリーダーはインデックス2（前0・中1・リーダー2） */
export const KC_LEADER_FORMATION_SLOT_INDEX = 2;

/** リーダー枠が埋まっていれば出撃・援軍の対象にできる */
export function isKcUnitReadyToDeploy(indices: [number, number, number]): boolean {
  return indices[KC_LEADER_FORMATION_SLOT_INDEX] >= 0;
}

export function countFilledFormationBodies(indices: [number, number, number]): number {
  return indices.filter((i) => i >= 0).length;
}

/** 前衛→中衛→リーダーの順で、埋まっている体の本拠スロットだけを配列化 */
export function formationBodyIndicesInSlotOrder(indices: [number, number, number]): number[] {
  const out: number[] = [];
  for (let s = 0; s < 3; s++) {
    const bi = indices[s];
    if (bi >= 0) out.push(bi);
  }
  return out;
}

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
  const serverCounts = getPlayerCardMonsterCounts(gameState);
  let counts =
    serverCounts.length >= homeTroops
      ? [...serverCounts]
      : [...serverCounts, ...Array(homeTroops - serverCounts.length).fill(DEFAULT_BODY_MONSTER_COUNT)];
  while (counts.length < homeTroops) {
    counts.push(DEFAULT_BODY_MONSTER_COUNT);
  }
  counts = counts
    .slice(0, homeTroops)
    .map((c) =>
      Math.max(MIN_MONSTER_COUNT_PER_CARD_SLOT, Math.min(MAX_MONSTER_COUNT_PER_CARD_SLOT, c)),
    );
  setBodyMonsterCounts(counts);

  const speeds = [...bodySpeeds];
  while (speeds.length < homeTroops) {
    speeds.push(Math.floor(Math.random() * 5) + 3);
  }
  const spd = speeds.slice(0, homeTroops);
  setBodySpeeds(spd);

  // 有効なユニット: 各indexが-1（空き）または 0..homeTroops-1
  let units = formedUnitsList.filter((u) =>
    u.indices.every((i) => i === -1 || (i >= 0 && i < homeTroops))
  );
  const deployableUnits = units.filter((u) => isKcUnitReadyToDeploy(u.indices));
  const nonDeployableUnits = units.filter((u) => !isKcUnitReadyToDeploy(u.indices));
  // 出撃可能ユニットが本拠体数を超える場合は末尾から削る（未完成は編集用に残す）
  let toKeep = deployableUnits;
  const bodiesUsed = (u: (typeof units)[number]) => countFilledFormationBodies(u.indices);
  while (toKeep.reduce((s, u) => s + bodiesUsed(u), 0) > homeTroops && toKeep.length > 0) {
    toKeep = toKeep.slice(0, -1);
  }
  units = [...toKeep, ...nonDeployableUnits].map((u) => ({
    ...u,
    ...recalcUnitStats(u.indices, counts, spd),
  }));
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
  const mcSlot = (i: number) =>
    Math.max(
      MIN_MONSTER_COUNT_PER_CARD_SLOT,
      Math.min(
        MAX_MONSTER_COUNT_PER_CARD_SLOT,
        monsterCounts[i] ?? DEFAULT_BODY_MONSTER_COUNT,
      ),
    );
  const monster_count = valid.reduce((s, i) => s + mcSlot(i), 0);
  const avgSpeed = valid.reduce((s, i) => s + (speeds[i] ?? DEFAULT_BODY_SPEED), 0) / valid.length;
  return { monster_count, avgSpeed };
}

/** 所持魔獣IDが本拠体インデックスに収まるものを優先し、足りなければ 0..から埋める */
function pickDefaultIndicesForOwned(): [number, number, number] | null {
  const troops = getHomeTroops();
  if (troops < BODIES_PER_UNIT) return null;
  const owned = getPlayerOwnedCards(gameState);
  const picks: number[] = [];
  const seen = new Set<number>();
  for (let slot = 0; slot < owned.length && slot < troops && picks.length < 3; slot++) {
    if (!seen.has(slot)) {
      seen.add(slot);
      picks.push(slot);
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
  const owned = getPlayerOwnedCards(gameState);
  const devCost = tri.reduce((s, i) => s + getCharacterStats(owned[i] ?? 0).cost, 0);
  if (devCost > cap + 0.0001) return;

  const { monster_count, avgSpeed } = recalcUnitStats(tri, bodyMonsterCounts, bodySpeeds);
  setFormedUnitsList([{
    id: `unit-${getNextFormedUnitId()}`,
    name: "ユニット1",
    indices: tri,
    monster_count,
    avgSpeed,
  }]);
}
