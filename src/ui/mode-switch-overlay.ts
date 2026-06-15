/**
 * モード切替中のローディングオーバーレイ
 */

import type { GameMode } from "../config";

let overlayEl: HTMLDivElement | null = null;
const MIN_DISPLAY_MS = 900;

function modeIcon(label: string): string {
  return `<svg class="mode-switch-icon" viewBox="0 0 64 64" width="64" height="64">
    <circle cx="32" cy="32" r="28" fill="rgba(201,168,76,0.06)" stroke="currentColor" stroke-width="1.5" opacity="0.8"/>
    <ellipse cx="32" cy="32" rx="18" ry="28" fill="none" stroke="currentColor" stroke-width="1" opacity="0.45"/>
    <ellipse cx="32" cy="32" rx="10" ry="28" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.3"/>
    <ellipse cx="32" cy="32" rx="24" ry="8" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.35"/>
    <ellipse cx="32" cy="32" rx="27" ry="4" fill="none" stroke="currentColor" stroke-width="0.6" opacity="0.2"/>
    <path d="M 4 18 Q 32 24 60 18" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.3"/>
    <path d="M 4 26 Q 32 32 60 26" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.35"/>
    <path d="M 4 32 Q 32 38 60 32" fill="none" stroke="currentColor" stroke-width="1" opacity="0.4"/>
    <path d="M 4 38 Q 32 44 60 38" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.35"/>
    <path d="M 4 46 Q 32 52 60 46" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.3"/>
    <path d="M 32 4 Q 40 32 32 60" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.25"/>
    <path d="M 32 4 Q 24 32 32 60" fill="none" stroke="currentColor" stroke-width="0.8" opacity="0.25"/>
    <circle cx="32" cy="32" r="0.6" fill="currentColor" opacity="0.5"/>
    <text x="32" y="36" text-anchor="middle" fill="currentColor" font-size="12" font-weight="bold" font-family="sans-serif" opacity="0.85">${label}</text>
  </svg>`;
}

function waitForMs(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function showModeSwitchOverlay(mode: GameMode): void {
  if (overlayEl) return;

  overlayEl = document.createElement("div");
  overlayEl.className = "mode-switch-overlay";
  overlayEl.innerHTML = `
    <div class="mode-switch-content">
      <div class="mode-switch-icon-wrap">${modeIcon(mode.toUpperCase())}</div>
      <div class="mode-switch-label">${mode.toUpperCase()}サーバーに接続中...</div>
      <div class="mode-switch-progress">
        <div class="mode-switch-progress-bar"></div>
      </div>
    </div>
  `;
  document.body.appendChild(overlayEl);
  overlayEl.classList.add("is-active");
}

export async function hideModeSwitchOverlay(): Promise<void> {
  if (!overlayEl) return;

  // 最低表示時間
  await waitForMs(MIN_DISPLAY_MS);

  overlayEl.classList.remove("is-active");
  overlayEl.classList.add("is-exiting");

  return new Promise<void>((resolve) => {
    const onEnd = () => {
      if (overlayEl) {
        overlayEl.remove();
        overlayEl = null;
      }
      resolve();
    };
    overlayEl?.addEventListener("transitionend", onEnd, { once: true });
    setTimeout(onEnd, 600);
  });
}
