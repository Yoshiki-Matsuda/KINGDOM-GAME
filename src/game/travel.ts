/**
 * 移動時間計算（サーバー `model/travel.rs` と同一式）
 */

export { getDistanceBetweenTerritories, getDistanceFromHome } from "./territories";
import { BASE_TRAVEL_TIME_PER_TILE } from "./characters";
export { BASE_TRAVEL_TIME_PER_TILE } from "./characters";

/** 移動時間（ミリ秒）。距離とユニット平均速さから計算。速さが高いほど短い */
export function getTravelTimeMs(distance: number, avgSpeed: number): number {
  if (distance <= 0 || avgSpeed <= 0) return 0;
  const refSpeed = 5;
  const secPerTile = BASE_TRAVEL_TIME_PER_TILE * (refSpeed / avgSpeed);
  return Math.max(0, Math.round(distance * secPerTile * 1000));
}

// クライアント側の移動タイマーは廃止（遠征はサーバー `marches` で管理）
export function startTravelIntervalIfNeeded(): void {
  // no-op: 後方互換
}
