/**
 * 編成ロジック — ユニットの検証・作成・解体
 */

import {
  bodyEnergies, bodySpeeds,
  formedUnitsList, setFormedUnitsList,
  getNextFormedUnitId,
  setBodyEnergies, setBodySpeeds,
  gameState,
} from "../store";
import { BODIES_PER_UNIT, DEFAULT_BODY_ENERGY, DEFAULT_BODY_SPEED } from "./characters";
import { HOME_TERRITORY_ID } from "../map-view";

/** 本拠地の体数を取得 */
export function getHomeTroops(): number {
  const t = gameState.territories.find((x) => x.id === HOME_TERRITORY_ID);
  return t?.troops ?? 0;
}

/**
 * 本拠地の体数が変わったら bodyEnergies/bodySpeeds を伸縮し、
 * 編成済みユニットを検証する
 */
export function validateFormedUnits(): void {
  const homeTroops = getHomeTroops();
  const energies = [...bodyEnergies];
  while (energies.length < homeTroops) {
    energies.push(DEFAULT_BODY_ENERGY);
  }
  setBodyEnergies(energies.slice(0, homeTroops));

  const speeds = [...bodySpeeds];
  while (speeds.length < homeTroops) {
    speeds.push(Math.floor(Math.random() * 5) + 3);
  }
  setBodySpeeds(speeds.slice(0, homeTroops));

  let units = formedUnitsList.filter((u) =>
    u.indices.every((i) => i >= 0 && i < homeTroops)
  );
  while (units.length * BODIES_PER_UNIT > homeTroops && units.length > 0) {
    units.pop();
  }
  setFormedUnitsList(units);
}

/** 開発用: 編成が0のとき1ユニット（キャラ0,1,2）を自動追加 */
export function ensureDevUnit(): void {
  validateFormedUnits();
  if (formedUnitsList.length === 0 && getHomeTroops() >= BODIES_PER_UNIT) {
    const e0 = bodyEnergies[0] ?? DEFAULT_BODY_ENERGY;
    const e1 = bodyEnergies[1] ?? DEFAULT_BODY_ENERGY;
    const e2 = bodyEnergies[2] ?? DEFAULT_BODY_ENERGY;
    const s0 = bodySpeeds[0] ?? DEFAULT_BODY_SPEED;
    const s1 = bodySpeeds[1] ?? DEFAULT_BODY_SPEED;
    const s2 = bodySpeeds[2] ?? DEFAULT_BODY_SPEED;
    setFormedUnitsList([{
      id: `unit-${getNextFormedUnitId()}`,
      name: "ユニット1",
      indices: [0, 1, 2],
      energy: e0 + e1 + e2,
      avgSpeed: (s0 + s1 + s2) / 3,
    }]);
  }
}
