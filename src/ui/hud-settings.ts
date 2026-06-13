/**
 * HUD 右上 — 設定（ログアウト・PVP/PVE 切替）
 */

import type { GameMode } from "../config";
import { USE_MOCK_STATE, authToken, gameMode, modeSwitchError, render, setModeSwitchError, setSelectedPvpServerId, getSelectedPvpServerId } from "../store";
import { switchGameMode } from "../network/mode-switch";
import { logout } from "../network/session";
import { pvpServerLabel, showPvpServerPickerDialog } from "./pvp-server-picker";

const GEAR_ICON = `<svg class="hud-settings-icon" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true"><path fill="currentColor" d="M12 15.5A3.5 3.5 0 0 1 8.5 12 3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5 3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97 0-.33-.03-.66-.07-1l2.11-1.63c.19-.15.24-.42.12-.64l-2-3.46c-.12-.22-.39-.31-.61-.22l-2.49 1c-.52-.39-1.06-.73-1.69-.98l-.37-2.65A.506.506 0 0 0 14 2h-4c-.25 0-.46.18-.5.42l-.37 2.65c-.63.25-1.17.59-1.69.98l-2.49-1c-.22-.09-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64L4.57 11c-.04.34-.07.67-.07 1 0 .33.03.65.07.97l-2.11 1.66c-.19.15-.25.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1.01c.52.4 1.06.74 1.69.99l.37 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.37-2.65c.63-.26 1.17-.59 1.69-.99l2.49 1.01c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.66Z"/></svg>`;

type SettingsCallbacks = {
  closeMenu: () => void;
  closeUnitSelect: () => void;
};

let settingsEl: HTMLDivElement;
let panelEl: HTMLDivElement;
let panelOpen = false;
let switching = false;
let callbacks: SettingsCallbacks = { closeMenu: () => {}, closeUnitSelect: () => {} };

function otherMode(mode: GameMode): GameMode {
  return mode === "pvp" ? "pve" : "pvp";
}

function setPanelOpen(open: boolean): void {
  panelOpen = open;
  const btn = settingsEl.querySelector<HTMLButtonElement>(".hud-settings-btn");
  if (open) {
    panelEl.classList.add("is-open");
    settingsEl.classList.add("is-open");
    btn?.setAttribute("aria-expanded", "true");
    updateHudSettings();
  } else {
    panelEl.classList.remove("is-open");
    settingsEl.classList.remove("is-open");
    btn?.setAttribute("aria-expanded", "false");
  }
}

function closePanel(): void {
  setPanelOpen(false);
}

function togglePanel(): void {
  setPanelOpen(!panelOpen);
}

async function handleSwitchMode(): Promise<void> {
  if (switching || !authToken) return;
  const next = otherMode(gameMode);
  closePanel();

  if (next === "pvp") {
    const serverId = await showPvpServerPickerDialog({
      initialServerId: getSelectedPvpServerId() ?? undefined,
    });
    if (!serverId) return;
    setSelectedPvpServerId(serverId);
  }

  switching = true;
  setModeSwitchError(null);
  updateHudSettings();
  const ok = await switchGameMode(next, callbacks);
  switching = false;
  if (ok) {
    render();
  } else {
    updateHudSettings();
  }
}

async function handleLogout(): Promise<void> {
  if (switching) return;
  closePanel();
  switching = true;
  updateHudSettings();
  await logout({ ...callbacks, render });
  switching = false;
}

export function initHudSettings(cbs: SettingsCallbacks): void {
  callbacks = cbs;
}

export function createHudSettingsElement(): HTMLDivElement {
  settingsEl = document.createElement("div");
  settingsEl.className = "hud-settings";

  settingsEl.innerHTML = `
    <button type="button" class="hud-settings-btn" aria-label="設定" aria-expanded="false" aria-haspopup="true">
      ${GEAR_ICON}
    </button>
    <div class="hud-settings-panel">
      <p class="hud-settings-current"></p>
      <button type="button" class="hud-settings-action" data-action="switch-mode"></button>
      <button type="button" class="hud-settings-action hud-settings-logout" data-action="logout">ログアウト</button>
      <p class="hud-settings-error" hidden></p>
    </div>
  `;

  panelEl = settingsEl.querySelector(".hud-settings-panel")!;
  const btn = settingsEl.querySelector<HTMLButtonElement>(".hud-settings-btn")!;
  const switchBtn = settingsEl.querySelector<HTMLButtonElement>('[data-action="switch-mode"]')!;
  const logoutBtn = settingsEl.querySelector<HTMLButtonElement>('[data-action="logout"]')!;

  btn.addEventListener("click", (event) => {
    event.stopPropagation();
    togglePanel();
  });

  switchBtn.addEventListener("click", () => void handleSwitchMode());
  logoutBtn.addEventListener("click", () => void handleLogout());

  document.addEventListener("click", (event) => {
    if (!panelOpen) return;
    if (!settingsEl.contains(event.target as Node)) {
      closePanel();
    }
  });

  return settingsEl;
}

export function updateHudSettings(): void {
  if (!settingsEl) return;

  if (USE_MOCK_STATE) {
    settingsEl.style.display = "none";
    return;
  }

  settingsEl.style.display = authToken ? "" : "none";

  const currentEl = settingsEl.querySelector<HTMLParagraphElement>(".hud-settings-current")!;
  const switchBtn = settingsEl.querySelector<HTMLButtonElement>('[data-action="switch-mode"]')!;
  const logoutBtn = settingsEl.querySelector<HTMLButtonElement>('[data-action="logout"]')!;
  const errorEl = settingsEl.querySelector<HTMLParagraphElement>(".hud-settings-error")!;

  if (gameMode === "pvp") {
    const label = pvpServerLabel(getSelectedPvpServerId() ?? "") ?? "";
    currentEl.textContent = label;
    currentEl.hidden = !label;
  } else {
    currentEl.hidden = true;
  }
  switchBtn.textContent = `${otherMode(gameMode).toUpperCase()} に切替`;
  switchBtn.disabled = switching || !authToken;
  logoutBtn.disabled = switching || !authToken;

  if (modeSwitchError) {
    errorEl.textContent = modeSwitchError;
    errorEl.hidden = false;
  } else {
    errorEl.hidden = true;
  }
}
