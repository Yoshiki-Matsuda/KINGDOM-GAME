/**
 * ステータス画面
 */

import { setCurrentScreen, render, gameState, ws } from "../store";
import {
  DEFAULT_PLAYER_ID,
  getPlayerData,
  getPlayerFacilities,
  getPlayerOwnedCards,
} from "../shared/game-state";
import { calculateFacilityBonuses } from "../game/facilities";
import { getCompletedFacilitiesMap } from "../game/facility-selectors";

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

  const facilities = getPlayerFacilities(gameState);
  const bonuses = calculateFacilityBonuses(getCompletedFacilitiesMap(facilities));
  const ownedCards = getPlayerOwnedCards(gameState);
  const player = getPlayerData(gameState, DEFAULT_PLAYER_ID);
  const explorations = player?.explorations ?? gameState.explorations ?? [];
  const explorationLv = player?.exploration_level ?? gameState.exploration_level ?? 1;
  const explorationScore = player?.exploration_score ?? gameState.exploration_score ?? 0;
  const res = gameState.resources ?? { food: 0, wood: 0, stone: 0, iron: 0, gold: 0 };
  const now = Date.now();

  const bonusRows = [
    bonuses.monsterBonus > 0 && `<div class="status-row">魔獣数上限: +${bonuses.monsterBonus}</div>`,
    bonuses.monsterPercent > 0 && `<div class="status-row">魔獣数増加: +${bonuses.monsterPercent}%</div>`,
    bonuses.speedBonus > 0 && `<div class="status-row">スピード: +${bonuses.speedBonus}</div>`,
    bonuses.skillPower > 0 && `<div class="status-row">スキル効果: +${bonuses.skillPower}%</div>`,
    bonuses.attackBonus > 0 && `<div class="status-row">攻撃力: +${bonuses.attackBonus}</div>`,
    bonuses.defenseBonus > 0 && `<div class="status-row">防御力: +${bonuses.defenseBonus}</div>`,
    bonuses.dropRate > 0 && `<div class="status-row">ドロップ率: +${bonuses.dropRate}%</div>`,
    bonuses.unitCapacity > 0 && `<div class="status-row">ユニット枠: +${bonuses.unitCapacity}</div>`,
  ].filter(Boolean);

  statusEl.innerHTML = `
    <div class="sub-screen-header">
      <h2>📊 ステータス</h2>
    </div>
    <div class="sub-screen-content">
      <div class="status-section">
        <h3>資源</h3>
        <div class="status-row">🌾 食料: ${res.food.toLocaleString()}</div>
        <div class="status-row">🪵 木材: ${res.wood.toLocaleString()}</div>
        <div class="status-row">🪨 石材: ${res.stone.toLocaleString()}</div>
        <div class="status-row">⛏️ 鉄: ${res.iron.toLocaleString()}</div>
        <div class="status-row">💰 ゴールド: ${res.gold.toLocaleString()}</div>
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
          explorations.length === 0
            ? '<div class="status-row dim">進行中の探索はありません。JSONで <code>start_exploration</code> を送るか今後UIから派遣予定。</div>'
            : explorations
                .map((m) => {
                  const done = now >= m.completes_at;
                  return `<div class="status-row exploration-row" data-mission="${m.mission_id}">
            ${m.territory_id} … ${done ? '<span class="explore-done">回収可能</span>' : `残り約 ${Math.max(0, Math.ceil((m.completes_at - now) / 1000))} 秒`}
            ${done ? `<button type="button" class="btn-collect-explore" data-mission="${m.mission_id}">回収</button>` : ""}
          </div>`;
                })
                .join("")
        }
      </div>
    </div>
  `;

  statusEl.querySelectorAll(".btn-collect-explore").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = (btn as HTMLButtonElement).dataset.mission;
      if (!id || ws?.readyState !== WebSocket.OPEN) return;
      ws.send(JSON.stringify({ action: "collect_exploration", mission_id: id }));
    });
  });
}
