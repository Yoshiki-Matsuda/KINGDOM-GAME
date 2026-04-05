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
import { isKcUnitReadyToDeploy } from "../game/formation";
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
    const hasFormedUnit = formedUnitsList.filter((u) => isKcUnitReadyToDeploy(u.indices)).length >= 1;
    if (!tid) {
      closeMenu();
      return;
    }
    if (!hasFormedUnit) {
      alert(
        "援軍に出せるユニットがありません。編成画面でリーダー枠にキャラを置いてユニットを編成してください。",
      );
      closeMenu();
      render();
      return;
    }
    if (ws?.readyState !== WebSocket.OPEN) {
      alert("サーバーに接続されていません。接続後にもう一度お試しください。");
      closeMenu();
      render();
      return;
    }
    closeMenu();
    showUnitSelect({ type: "deploy", territoryId: tid });
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
    const hasFormedUnit = formedUnitsList.filter((u) => isKcUnitReadyToDeploy(u.indices)).length >= 1;
    if (!toId) {
      alert("攻撃先のマスが不正です。");
      closeMenu();
      render();
      return;
    }
    if (!fromId) {
      alert(
        "このマスに隣接する自領がありません。本拠や占領済みマスの隣から攻撃してください。自領マスで「攻撃」を選んでから隣の敵／中立マスを攻撃することもできます。",
      );
      closeMenu();
      render();
      return;
    }
    if (!hasFormedUnit) {
      alert(
        "攻撃に出せるユニットがありません。編成画面でリーダー枠にキャラを置いてから、もう一度お試しください。",
      );
      closeMenu();
      render();
      return;
    }
    if (ws?.readyState !== WebSocket.OPEN) {
      alert("サーバーに接続されていません。接続後にもう一度お試しください。");
      closeMenu();
      render();
      return;
    }
    closeMenu();
    showUnitSelect({ type: "attack", fromId, toId });
    return;
  }
}
