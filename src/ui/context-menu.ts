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
import { showUnitSelect } from "./unit-select";
import { showFormationScreen } from "./formation-screen";
import { showHomeScreen } from "./home-screen";

let menuEl: HTMLDivElement;
let ruinTimerId: number | null = null;

/** 難易度のラベル */
function getDifficultyLabel(difficulty: string): string {
  switch (difficulty) {
    case "normal": return "★";
    case "rare": return "★★";
    case "legendary": return "★★★";
    default: return difficulty;
  }
}

/** 残り時間をフォーマット */
function formatRuinTimeLeft(expiresAt: number): string {
  const now = Date.now();
  const remaining = Math.max(0, expiresAt - now);
  const totalSec = Math.floor(remaining / 1000);
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  return `${min}:${sec.toString().padStart(2, "0")}`;
}

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
    menuEl.innerHTML = `
      <button type="button" data-action="enter">入る</button>
      <button type="button" data-action="formation">編成</button>
    `;
  } else if (t.ruin) {
    // 遺跡マス
    const ruin = t.ruin;
    const attackable = isAttackable(gameState, territoryId);
    const timeLeftHtml = ruin.expires_at ? formatRuinTimeLeft(ruin.expires_at) : "";

    const enemyNames = ruin.enemy_names ?? ruin.enemies;
    menuEl.innerHTML = `
      <div class="context-menu-ruin">
        <div class="ruin-title">${ruin.formation_name}</div>
        <div class="ruin-difficulty ruin-${ruin.difficulty}">${getDifficultyLabel(ruin.difficulty)}</div>
        ${timeLeftHtml ? `<div class="ruin-time-left" data-expires-at="${ruin.expires_at}">残り ${timeLeftHtml}</div>` : ""}
        <div class="ruin-enemies">
          ${enemyNames.map((name) => `<span class="ruin-enemy">${name}</span>`).join("")}
        </div>
      </div>
      ${attackable ? `<button type="button" data-action="attack" data-to="${territoryId}">挑戦</button>` : ""}
    `;

    // 残り時間を1秒ごとに更新
    if (ruin.expires_at) {
      startRuinTimer();
    }
  } else {
    const canDeploy = canReceiveReinforcement(gameState, t);

    if (canDeploy) {
      const isOwn = t.owner_id === "player";
      menuEl.innerHTML = `
      <button type="button" data-action="deploy" data-territory="${territoryId}">援軍</button>
      ${isOwn ? `<button type="button" data-action="attack-from" data-territory="${territoryId}">攻撃</button>` : ""}
    `;
    } else {
      const statusText = t.owner_id ? "敵占領" : "中立";
      const attackable = isAttackable(gameState, territoryId);
      menuEl.innerHTML = `
      <div class="context-menu-info">Lv.${t.level} ${t.name}（${statusText}）</div>
      ${attackable ? `<button type="button" data-action="attack" data-to="${territoryId}">攻撃</button>` : ""}
    `;
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
  if (action === "formation") {
    closeMenu();
    showFormationScreen();
    return;
  }
  if (action === "enter") {
    closeMenu();
    showHomeScreen();
  }
}
