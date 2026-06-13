/**
 * ステータス画面
 */

import { setCurrentScreen, render, gameState, getLocalPlayerId } from "../store";
import {
  getPlayerData,
  getPlayerFacilities,
  getPlayerMarches,
  getPlayerOwnedCards,
  getPlayerResources,
} from "../shared/game-state";
import { calculateFacilityBonuses } from "../game/facilities";
import { getCompletedFacilitiesMap } from "../game/facility-selectors";
import { renderScreenHeaderTitle } from "./screen-header";
import { renderResourceValueHtml } from "./resource-display";

let statusEl: HTMLDivElement | null = null;

export function createStatusElement(): HTMLDivElement {
  const el = document.createElement("div");
  el.className = "sub-screen status-screen";
  el.style.display = "none";
  statusEl = el;
  return el;
}

export function showStatusScreen(): void {
  setCurrentScreen("status");
  render();
}

export function renderStatus(): void {
  if (!statusEl || !gameState) return;

  const facilities = getPlayerFacilities(gameState, getLocalPlayerId());
  const bonuses = calculateFacilityBonuses(getCompletedFacilitiesMap(facilities));
  const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const player = getPlayerData(gameState, getLocalPlayerId());
  const marches = getPlayerMarches(gameState, getLocalPlayerId());
  const explorationLv = player?.exploration_level ?? 1;
  const explorationScore = player?.exploration_score ?? 0;
  const res = getPlayerResources(gameState, getLocalPlayerId());
  const now = Date.now();

  const bonusRows = [
    bonuses.monsterBonus > 0 && `<div class="status-row">魔獣数上限: +${bonuses.monsterBonus}</div>`,
    bonuses.monsterPercent > 0 && `<div class="status-row">魔獣数増加: +${bonuses.monsterPercent}%</div>`,
    bonuses.speedBonus > 0 && `<div class="status-row">速さ: +${bonuses.speedBonus}</div>`,
    bonuses.skillPower > 0 && `<div class="status-row">スキル効果: +${bonuses.skillPower}%</div>`,
    bonuses.attackBonus > 0 && `<div class="status-row">攻撃力: +${bonuses.attackBonus}</div>`,
    bonuses.defenseBonus > 0 && `<div class="status-row">防御力: +${bonuses.defenseBonus}</div>`,
    bonuses.dropRate > 0 && `<div class="status-row">ドロップ率: +${bonuses.dropRate}%</div>`,
    bonuses.unitCapacity > 0 && `<div class="status-row">ユニット枠: +${bonuses.unitCapacity}</div>`,
  ].filter(Boolean);

  statusEl.innerHTML = `
    <div class="sub-screen-header">
      <h2>${renderScreenHeaderTitle("status", "ステータス")}</h2>
    </div>
    <div class="sub-screen-content">
      <div class="status-section">
        <h3>資源</h3>
        <div class="status-row">${renderResourceValueHtml("food", res.food)}</div>
        <div class="status-row">${renderResourceValueHtml("wood", res.wood)}</div>
        <div class="status-row">${renderResourceValueHtml("stone", res.stone)}</div>
        <div class="status-row">${renderResourceValueHtml("iron", res.iron)}</div>
        <div class="status-row">${renderResourceValueHtml("gold", res.gold)}</div>
      </div>
      <div class="status-section">
        <h3>施設ボーナス</h3>
        ${bonusRows.length > 0
          ? bonusRows.join('\n        ')
          : '<div class="status-row dim">施設を建設してボーナスを獲得しましょう</div>'
        }
      </div>
      <div class="status-section">
        <h3>所持魔獣</h3>
        <div class="status-row">魔獣枠数: ${ownedCards.length}</div>
      </div>
      <div class="status-section">
        <h3>探索</h3>
        <div class="status-row">探索レベル: ${explorationLv}（スコア ${explorationScore}）</div>
        ${
          marches.length === 0
            ? '<div class="status-row dim">進行中の遠征はありません。</div>'
            : marches
                .map((m) => {
                  const secLeft = Math.max(0, Math.ceil((m.arrives_at - now) / 1000));
                  const kindLabel =
                    m.kind === "attack" ? "攻撃" :
                    m.kind === "explore" ? "探索" :
                    m.kind === "deploy" ? "援軍" : "帰還";
                  return `<div class="status-row exploration-row">
            ${kindLabel}: ${m.from_territory_id} → ${m.to_territory_id} … 残り約 ${secLeft} 秒
          </div>`;
                })
                .join("")
        }
      </div>
    </div>
  `;
}
