/**
 * コンテキストメニュー — 領地クリックメニュー
 */

import type { Territory } from "../store";
import {
  gameState, ws,
  attackSourceId, setAttackSourceId,
  formedUnitsList,
  render,
} from "../store";
import { canReceiveReinforcement, isAttackable, getAdjacentAttackSource, isHomeTerritory } from "../game/combat";
import {
  formatRuinTimeLeft,
  renderNeutralTerritoryMenu,
  renderOwnedTerritoryMenu,
  renderRuinContextMenu,
} from "../context-menu-view";
import { showUnitSelect } from "./unit-select";

let menuEl: HTMLDivElement;
let ruinTimerId: number | null = null;

/** 残り時間を1秒ごとに更新 */
function startRuinTimer(): void {
  stopRuinTimer();
  ruinTimerId = window.setInterval(() => {
    const el = menuEl.querySelector(".ruin-time-left") as HTMLElement | null;
    if (!el) {
      stopRuinTimer();
      return;
    }
    const expiresAt = parseInt(el.dataset.expiresAt ?? "0", 10);
    if (expiresAt <= Date.now()) {
      el.textContent = "消滅！";
      el.classList.add("expired");
      stopRuinTimer();
    } else {
      el.textContent = `残り ${formatRuinTimeLeft(expiresAt)}`;
    }
  }, 1000);
}

function stopRuinTimer(): void {
  if (ruinTimerId !== null) {
    clearInterval(ruinTimerId);
    ruinTimerId = null;
  }
}

export function createMenuElement(): HTMLDivElement {
  menuEl = document.createElement("div");
  menuEl.className = "context-menu";
  menuEl.hidden = true;
  menuEl.addEventListener("click", onMenuClick);
  return menuEl;
}

export function closeMenu(): void {
  menuEl.hidden = true;
  stopRuinTimer();
}

export function showMenuAt(x: number, y: number, territoryId: string, territory: Territory): void {
  const t = gameState.territories.find((x) => x.id === territoryId) ?? territory;

  if (isHomeTerritory(territoryId)) {
    menuEl.hidden = true;
    return;
  }
  if (t.ruin) {
    // 遺跡マス
    const attackable = isAttackable(gameState, territoryId);
    menuEl.innerHTML = renderRuinContextMenu(territoryId, t, attackable);

    // 残り時間を1秒ごとに更新
    if (t.ruin.expires_at) {
      startRuinTimer();
    }
  } else {
    const canDeploy = canReceiveReinforcement(gameState, t);

    if (canDeploy) {
      menuEl.innerHTML = renderOwnedTerritoryMenu(territoryId, t);
    } else {
      const attackable = isAttackable(gameState, territoryId);
      menuEl.innerHTML = renderNeutralTerritoryMenu(territoryId, t, attackable);
    }
  }

  menuEl.hidden = false;
  const OFFSET = 12;
  const menuW = menuEl.offsetWidth || 160;
  const menuH = menuEl.offsetHeight || 120;
  const padding = 8;
  let left = x + OFFSET;
  let top = y + OFFSET;
  if (left + menuW > window.innerWidth - padding) left = x - menuW - OFFSET;
  if (top + menuH > window.innerHeight - padding) top = y - menuH - OFFSET;
  if (left < padding) left = padding;
  if (top < padding) top = padding;
  menuEl.style.left = `${left}px`;
  menuEl.style.top = `${top}px`;
}

function onMenuClick(e: MouseEvent): void {
  const btn = (e.target as HTMLElement).closest("button");
  if (!btn || !menuEl.contains(btn)) return;
  e.preventDefault();
  e.stopPropagation();
  const action = btn.dataset.action;

  if (action === "deploy") {
    const tid = btn.dataset.territory ?? null;
    const hasFormedUnit = formedUnitsList.filter((u) => u.indices.every((i) => i >= 0)).length >= 1;
    if (tid && ws?.readyState === WebSocket.OPEN && hasFormedUnit) {
      closeMenu();
      showUnitSelect({ type: "deploy", territoryId: tid });
    } else if (!tid) {
      closeMenu();
    } else if (!hasFormedUnit) {
      closeMenu();
    } else {
      closeMenu();
      render();
    }
    return;
  }
  if (action === "attack-from") {
    setAttackSourceId(btn.dataset.territory ?? null);
    closeMenu();
    render();
    return;
  }
  if (action === "attack") {
    const toId = btn.dataset.to ?? null;
    const fromId = attackSourceId ?? (toId ? getAdjacentAttackSource(gameState, toId) : null);
    const hasFormedUnit = formedUnitsList.filter((u) => u.indices.every((i) => i >= 0)).length >= 1;
    if (ws?.readyState === WebSocket.OPEN && fromId && toId && hasFormedUnit) {
      closeMenu();
      showUnitSelect({ type: "attack", fromId, toId });
    } else {
      if (!toId) console.warn("[攻撃] 攻撃先がありません");
      else if (!fromId) console.warn("[攻撃] 隣接する自領がありません");
      else if (ws?.readyState !== WebSocket.OPEN) console.warn("[攻撃] サーバー未接続");
      else if (!hasFormedUnit) console.warn("[攻撃] 編成されたユニットがありません（3体で1ユニットに編成されると出撃できます）");
      closeMenu();
      render();
    }
    return;
  }
}
