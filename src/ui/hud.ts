/**
 * HUD描画 — 接続状態・施設ボーナス
 */

import {
  USE_MOCK_STATE, connectionStatus, gameState, getLocalPlayerId,
} from "../store";
import { getFacilityBonusesForState } from "../game/facility-selectors";
import { getPlayerResources } from "../shared/game-state";
import { renderResourcesHtml } from "./resource-display";
import { syncResourceChangeFlashes } from "./resource-flash";

export { renderResourcesHtml } from "./resource-display";

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

  const bonuses = getFacilityBonusesForState(gameState);

  const bonusTexts: string[] = [];
  if (bonuses.monsterBonus > 0) bonusTexts.push(`M+${bonuses.monsterBonus}`);
  if (bonuses.monsterPercent > 0) bonusTexts.push(`M+${bonuses.monsterPercent}%`);
  if (bonuses.speedBonus > 0) bonusTexts.push(`速さ+${bonuses.speedBonus}`);
  if (bonuses.dropRate > 0) bonusTexts.push(`DROP+${bonuses.dropRate}%`);

  const bonusDisplay = bonusTexts.length > 0
    ? `<span class="hud-bonus">${bonusTexts.join(" ")}</span>`
    : "";

  const resDisplay = renderResourcesHtml();

  hudEl.innerHTML = `
    <span class="hud-status" data-status="${USE_MOCK_STATE ? "mock" : connectionStatus}">${statusText}</span>
    ${resDisplay}
    ${bonusDisplay}
  `;

  const resBar = hudEl.querySelector(".hud-resources");
  if (resBar) {
    syncResourceChangeFlashes(resBar, getPlayerResources(gameState, getLocalPlayerId()));
  }
}
