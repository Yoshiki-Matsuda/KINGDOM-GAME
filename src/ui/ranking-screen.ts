/**
 * ランキング画面
 */

import { setCurrentScreen, render } from "../store";

let rankingEl: HTMLDivElement | null = null;

export function createRankingElement(): HTMLDivElement {
  const el = document.createElement("div");
  el.className = "sub-screen ranking-screen";
  el.style.display = "none";
  rankingEl = el;
  return el;
}

export function showRankingScreen(): void {
  setCurrentScreen("ranking");
  render();
}

export function renderRanking(): void {
  if (!rankingEl) return;

  rankingEl.innerHTML = `
    <div class="sub-screen-header">
      <h2>🏆 ランキング</h2>
    </div>
    <div class="sub-screen-content">
      <div class="sub-screen-empty">ランキング機能は準備中です</div>
    </div>
  `;
}
