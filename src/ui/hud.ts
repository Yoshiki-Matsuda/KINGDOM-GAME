/**
 * HUD描画 — 接続状態・施設ボーナス
 */

import {
  USE_MOCK_STATE, connectionStatus, gameState, getLocalPlayerId,
} from "../store";
import { getFacilityBonusesForState } from "../game/facility-selectors";
import { getPlayerResources, type Resources } from "../shared/game-state";
import { renderResourcesHtml, formatResourceAmount } from "./resource-display";
import { syncResourceChangeFlashes } from "./resource-flash";

export { renderResourcesHtml } from "./resource-display";

let hudEl: HTMLDivElement;

export function createHudElement(): HTMLDivElement {
  hudEl = document.createElement("div");
  hudEl.className = "hud";
  return hudEl;
}

/** 資源バー内の数値をDOM再生成なしで更新 */
function updateResourceValues(resBar: HTMLElement, res: Resources): void {
  for (const type of ["food", "wood", "stone", "iron", "gold"] as const) {
    const amountEl = resBar.querySelector<HTMLElement>(`[data-resource-type="${type}"] .resource-amount`);
    if (amountEl) {
      amountEl.textContent = formatResourceAmount(res[type]);
    }
  }
}

export function renderHud(): void {
  const currentRes = getPlayerResources(gameState, getLocalPlayerId());

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

  const resBar = hudEl.querySelector<HTMLElement>(".hud-resources");
  if (resBar) {
    // 資源バーのDOMを再生成せず、数値のみを更新（アニメーションを壊さない）
    updateResourceValues(resBar, currentRes);
    syncResourceChangeFlashes(resBar, currentRes);

    // ステータスとボーナスは更新
    const statusEl = hudEl.querySelector(".hud-status");
    if (statusEl) {
      statusEl.textContent = statusText;
      statusEl.setAttribute("data-status", USE_MOCK_STATE ? "mock" : connectionStatus);
    }

    // ボーナスの更新
    const existingBonus = hudEl.querySelector(".hud-bonus");
    if (bonusTexts.length > 0) {
      if (existingBonus) {
        existingBonus.textContent = bonusTexts.join(" ");
      } else {
        hudEl.insertAdjacentHTML("beforeend", bonusDisplay);
      }
    } else if (existingBonus) {
      existingBonus.remove();
    }
  } else {
    const resDisplay = renderResourcesHtml();
    hudEl.innerHTML = `
      <span class="hud-status" data-status="${USE_MOCK_STATE ? "mock" : connectionStatus}">${statusText}</span>
      ${resDisplay}
      ${bonusDisplay}
    `;

    const newResBar = hudEl.querySelector<HTMLElement>(".hud-resources");
    if (newResBar) {
      syncResourceChangeFlashes(newResBar, currentRes);
    }
  }
}
