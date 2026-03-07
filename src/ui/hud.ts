/**
 * HUD描画 — 接続状態・施設ボーナス
 */

import {
  USE_MOCK_STATE, connectionStatus, gameState,
} from "../store";
import { calculateFacilityBonuses, type FacilityId } from "../game/facilities";

let hudEl: HTMLDivElement;

export function createHudElement(): HTMLDivElement {
  hudEl = document.createElement("div");
  hudEl.className = "hud";
  return hudEl;
}

export function renderHud(): void {
  const statusText = USE_MOCK_STATE
    ? "開発用マスデータ"
    : connectionStatus === "online"
      ? "オンライン"
      : "オフライン";

  // 施設ボーナス計算
  const builtFacilities = new Map<FacilityId, number>();
  for (const f of gameState.facilities ?? []) {
    if (!f.build_complete_at || f.build_complete_at <= Date.now()) {
      builtFacilities.set(f.facility_id as FacilityId, f.level);
    }
  }
  const bonuses = calculateFacilityBonuses(builtFacilities);

  const bonusTexts: string[] = [];
  if (bonuses.energyBonus > 0) bonusTexts.push(`EN+${bonuses.energyBonus}`);
  if (bonuses.energyPercent > 0) bonusTexts.push(`EN+${bonuses.energyPercent}%`);
  if (bonuses.speedBonus > 0) bonusTexts.push(`SPD+${bonuses.speedBonus}`);
  if (bonuses.dropRate > 0) bonusTexts.push(`DROP+${bonuses.dropRate}%`);

  const bonusDisplay = bonusTexts.length > 0 
    ? `<span class="hud-bonus">${bonusTexts.join(" ")}</span>` 
    : "";

  hudEl.innerHTML = `
    <span class="hud-status" data-status="${USE_MOCK_STATE ? "mock" : connectionStatus}">${statusText}</span>
    ${bonusDisplay}
  `;
}
