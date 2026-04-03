/**
 * カードパック画面（ガチャ/カードドロー）
 */

import { setCurrentScreen, render } from "../store";

let packEl: HTMLDivElement | null = null;

export function createPackElement(): HTMLDivElement {
  const el = document.createElement("div");
  el.className = "sub-screen pack-screen";
  el.style.display = "none";
  packEl = el;
  return el;
}

export function showPackScreen(): void {
  setCurrentScreen("pack");
  render();
}

export function renderPack(): void {
  if (!packEl) return;

  packEl.innerHTML = `
    <div class="sub-screen-header">
      <h2>🃏 カードパック</h2>
    </div>
    <div class="sub-screen-content">
      <div class="sub-screen-empty">カードパック機能は準備中です</div>
      <div class="pack-info">
        <p>戦闘でドロップしたカードやフリマで購入したカードが使えます。</p>
      </div>
    </div>
  `;
}
