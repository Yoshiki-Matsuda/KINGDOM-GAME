/**
 * 魔獣パック画面（ガチャ／ドロー）
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
      <h2>🃏 魔獣パック</h2>
    </div>
    <div class="sub-screen-content">
      <div class="sub-screen-empty">魔獣パック機能は準備中です</div>
      <div class="pack-info">
        <p>戦闘でドロップした魔獣やフリマで購入した魔獣が使えます。</p>
      </div>
    </div>
  `;
}
