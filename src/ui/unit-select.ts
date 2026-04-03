/**
 * ユニット選択オーバーレイ
 */

import type { PendingUnitAction } from "../store";
import {
  ws,
  formedUnitsList,
  bodyMonsterCounts,
  bodySpeeds,
  pendingUnitAction, setPendingUnitAction,
  setAttackSourceId,
  render,
  getNextTravelingId,
} from "../store";
import type { TravelingUnit } from "../store";
import { BODIES_PER_UNIT, DEFAULT_BODY_MONSTER_COUNT, DEFAULT_BODY_SPEED, getBodyDisplayName, getCharacterSkillData, getCharacterStats } from "../game/characters";
import { gameState } from "../store";
import { getPlayerOwnedCards } from "../shared/game-state";
import { getDistanceFromHome, getTravelTimeMs, startTravelIntervalIfNeeded } from "../game/travel";
import { closeMenu } from "./context-menu";
import { showFormationScreen } from "./formation-screen";
import { appendTravelingUnit } from "../store-actions";
import {
  getUnitSelectSnapshot,
  renderAvailableUnits,
  renderReturningUnits,
} from "../unit-select-view";

let unitSelectEl: HTMLDivElement;
let unitSelectEscapeHandler: ((e: KeyboardEvent) => void) | null = null;

export function createUnitSelectElement(): HTMLDivElement {
  unitSelectEl = document.createElement("div");
  unitSelectEl.className = "unit-select-overlay";
  unitSelectEl.innerHTML = `
    <div class="unit-select-modal">
      <div class="unit-select-title" data-unit-title>ユニットを選択</div>
      <div class="unit-select-panel-form" data-unit-panel-form style="display:none">
        <div class="unit-select-troops" data-unit-troops>編成済みユニットから送るユニットを選んでください</div>
        <div class="unit-select-unit-list" data-unit-list></div>
        <div class="unit-select-returning" data-unit-returning style="display:none">
          <div class="unit-select-returning-title">帰還中</div>
          <div class="unit-select-returning-list" data-unit-returning-list></div>
        </div>
        <div class="unit-select-actions">
          <button type="button" class="secondary" data-unit-cancel>キャンセル</button>
          <button type="button" class="primary" data-unit-confirm>決定</button>
        </div>
      </div>
      <div class="unit-select-panel-empty" data-unit-panel-empty style="display:none">
        <p class="unit-select-empty-msg">編成されたユニットがありません。編成画面でキャラ3体を選んでユニットを編成してください。</p>
        <div class="unit-select-returning unit-select-returning--empty" data-unit-returning-empty style="display:none">
          <div class="unit-select-returning-title">帰還中</div>
          <div class="unit-select-returning-list" data-unit-returning-list-empty></div>
        </div>
        <button type="button" class="primary" data-unit-open-formation>編成画面を開く</button>
        <button type="button" class="secondary" data-unit-cancel-empty>キャンセル</button>
      </div>
    </div>
  `;
  setupUnitSelect();
  return unitSelectEl;
}

export function showUnitSelect(pending: PendingUnitAction): void {
  if (!pending) return;
  setPendingUnitAction(pending);
  const titleEl = unitSelectEl.querySelector("[data-unit-title]")!;
  const panelForm = unitSelectEl.querySelector("[data-unit-panel-form]") as HTMLElement;
  const panelEmpty = unitSelectEl.querySelector("[data-unit-panel-empty]") as HTMLElement;
  const troopsEl = unitSelectEl.querySelector("[data-unit-troops]")!;
  const listEl = unitSelectEl.querySelector("[data-unit-list]")!;

  if (pending.type === "attack") {
    titleEl.textContent = "攻撃するユニットを選択";
  } else {
    titleEl.textContent = "援軍するユニットを選択";
  }

  const now = Date.now();
  const { completeUnits, availableUnits, returningUnits } = getUnitSelectSnapshot(now);

  if (availableUnits.length === 0) {
    panelForm.style.display = "none";
    panelEmpty.style.display = "block";
    const emptyMsg = unitSelectEl.querySelector(".unit-select-empty-msg")!;
    emptyMsg.textContent =
      completeUnits.length > 0
        ? "選択できるユニットがありません。出撃中・帰還中のユニットは到着までお待ちください。"
        : "編成されたユニットがありません。編成画面でキャラ3体を選んでユニットを編成してください。";
    const returningEmpty = unitSelectEl.querySelector("[data-unit-returning-empty]") as HTMLElement;
    const listEmpty = unitSelectEl.querySelector("[data-unit-returning-list-empty]")!;
    if (returningUnits.length > 0) {
      returningEmpty.style.display = "block";
      renderReturningUnits(listEmpty, now, returningUnits);
    } else {
      returningEmpty.style.display = "none";
    }
  } else {
    panelEmpty.style.display = "none";
    panelForm.style.display = "block";
    troopsEl.textContent = `編成済みユニットから送る1ユニットを選んでください`;
    renderAvailableUnits(listEl, availableUnits);
    const returningBlock = unitSelectEl.querySelector("[data-unit-returning]") as HTMLElement;
    const returningListEl = unitSelectEl.querySelector("[data-unit-returning-list]")!;
    if (returningUnits.length > 0) {
      returningBlock.style.display = "block";
      renderReturningUnits(returningListEl, now, returningUnits);
    } else {
      returningBlock.style.display = "none";
    }
  }

  unitSelectEl.classList.add("is-open");
  bindUnitSelectEscape();
}

/**
 * 編成スロットの体インデックス（≒そのマスのキャラ定義ID）に対応する `owned_cards` 配列上のインデックス。
 * 完全一致が無い（例: 編成0,1,2なのに所持がゴブリンだけ）でも、負担分散で割り当てて攻撃可能にする。
 */
function resolveOwnedCardIndices(bodyIndices: [number, number, number], owned: number[]): number[] | null {
  if (owned.length === 0) return null;

  const usage = new Map<number, number>();
  const out: number[] = [];

  for (let slot = 0; slot < 3; slot++) {
    const bi = bodyIndices[slot];
    let bestJ = -1;
    let bestUses = Infinity;

    for (let j = 0; j < owned.length; j++) {
      if (owned[j] !== bi) continue;
      const u = usage.get(j) ?? 0;
      if (u < bestUses) {
        bestUses = u;
        bestJ = j;
      }
    }

    if (bestJ < 0) {
      for (let j = 0; j < owned.length; j++) {
        const u = usage.get(j) ?? 0;
        if (bestJ < 0 || u < bestUses) {
          bestUses = u;
          bestJ = j;
        }
      }
    }

    usage.set(bestJ, (usage.get(bestJ) ?? 0) + 1);
    out.push(bestJ);
  }

  return out;
}

export function closeUnitSelect(): void {
  setPendingUnitAction(null);
  unitSelectEl.classList.remove("is-open");
  unbindUnitSelectEscape();
}

/** ユニット選択が開いているとき、選択可能リスト・帰還中リストを現在状態で更新する（render のたびに呼ぶ） */
export function updateUnitSelectReturningList(): void {
  if (!unitSelectEl.classList.contains("is-open")) return;
  const now = Date.now();
  const { completeUnits, availableUnits, returningUnits } = getUnitSelectSnapshot(now);

  const panelForm = unitSelectEl.querySelector("[data-unit-panel-form]") as HTMLElement;
  const panelEmpty = unitSelectEl.querySelector("[data-unit-panel-empty]") as HTMLElement;
  const listEl = unitSelectEl.querySelector("[data-unit-list]")!;
  const troopsEl = unitSelectEl.querySelector("[data-unit-troops]")!;
  const returningBlock = unitSelectEl.querySelector("[data-unit-returning]") as HTMLElement;
  const returningListEl = unitSelectEl.querySelector("[data-unit-returning-list]")!;
  const returningEmpty = unitSelectEl.querySelector("[data-unit-returning-empty]") as HTMLElement;
  const listEmpty = unitSelectEl.querySelector("[data-unit-returning-list-empty]")!;

  if (availableUnits.length === 0) {
    panelForm.style.display = "none";
    panelEmpty.style.display = "block";
    const emptyMsg = unitSelectEl.querySelector(".unit-select-empty-msg")!;
    emptyMsg.textContent =
      completeUnits.length > 0
        ? "選択できるユニットがありません。出撃中・帰還中のユニットは到着までお待ちください。"
        : "編成されたユニットがありません。編成画面でキャラ3体を選んでユニットを編成してください。";
    if (returningUnits.length > 0) {
      if (returningEmpty) {
        returningEmpty.style.display = "block";
        renderReturningUnits(listEmpty, now, returningUnits);
      }
    } else {
      if (returningEmpty) returningEmpty.style.display = "none";
    }
  } else {
    panelEmpty.style.display = "none";
    panelForm.style.display = "block";
    troopsEl.textContent = "編成済みユニットから送る1ユニットを選んでください";
    renderAvailableUnits(listEl, availableUnits);
    if (returningUnits.length > 0) {
      if (returningBlock) {
        returningBlock.style.display = "block";
        renderReturningUnits(returningListEl, now, returningUnits);
      }
    } else {
      if (returningBlock) returningBlock.style.display = "none";
    }
    if (returningEmpty) returningEmpty.style.display = "none";
  }
}

function setupUnitSelect(): void {
  const confirmBtn = unitSelectEl.querySelector<HTMLButtonElement>("[data-unit-confirm]")!;
  const cancelBtn = unitSelectEl.querySelector<HTMLButtonElement>("[data-unit-cancel]")!;
  const openFormationBtn = unitSelectEl.querySelector<HTMLButtonElement>("[data-unit-open-formation]")!;
  const cancelEmptyBtn = unitSelectEl.querySelector<HTMLButtonElement>("[data-unit-cancel-empty]")!;

  confirmBtn.addEventListener("click", () => {
    const pending = pendingUnitAction;
    if (!pending || ws?.readyState !== WebSocket.OPEN) {
      closeUnitSelect();
      render();
      return;
    }
    const listEl = unitSelectEl.querySelector("[data-unit-list]")!;
    const checked = listEl.querySelector<HTMLInputElement>('input[name="unit-select-one"]:checked');
    const selectedId = checked?.dataset.unitId ?? null;
    if (!selectedId) {
      closeUnitSelect();
      render();
      return;
    }
    const count = 1 * BODIES_PER_UNIT;
    const unit = formedUnitsList.find((u) => u.id === selectedId);
    if (!unit) {
      closeUnitSelect();
      render();
      return;
    }
    const monstersPerBody = unit.indices.map((i) => bodyMonsterCounts[i] ?? DEFAULT_BODY_MONSTER_COUNT);
    const speedPerBody = unit.indices.map((i) => bodySpeeds[i] ?? DEFAULT_BODY_SPEED);
    const bodyNames = unit.indices.map((i) => getBodyDisplayName(i));
    const skillsPerBody = unit.indices.map((i) => getCharacterSkillData(i));
    const statsPerBody = unit.indices.map((i) => {
      const s = getCharacterStats(i);
      return {
        monster_count: s.monster_count,
        speed: s.speed,
        attack: s.attack,
        intelligence: s.intelligence,
        defense: s.defense,
        magic_defense: s.magicDefense,
        range: s.range,
        cost: s.cost,
        occupation_power: s.occupationPower,
      };
    });
    const owned = getPlayerOwnedCards(gameState);
    const ownedCardIndices = resolveOwnedCardIndices(unit.indices, owned);
    if (!ownedCardIndices) {
      console.warn("[kingdom] 所持カードが0枚のため出撃できません。", { unitIndices: unit.indices });
      closeUnitSelect();
      render();
      return;
    }
    const targetId = pending.type === "attack" ? pending.toId : pending.territoryId;
    const distance = getDistanceFromHome(targetId);
    const travelTimeMs = getTravelTimeMs(distance, unit.avgSpeed);

    const traveling: TravelingUnit = {
      id: `travel-${getNextTravelingId()}`,
      unitId: unit.id,
      unitName: unit.name,
      actionType: pending.type === "attack" ? "attack" : "deploy",
      targetId,
      fromId: pending.type === "attack" ? pending.fromId : undefined,
      count,
      monstersPerBody,
      speedPerBody,
      bodyNames,
      skillsPerBody,
      statsPerBody,
      ownedCardIndices,
      departureTime: Date.now(),
      arrivalTime: Date.now() + travelTimeMs,
    };
    appendTravelingUnit(traveling);
    startTravelIntervalIfNeeded();

    if (pending.type === "attack") setAttackSourceId(null);
    closeUnitSelect();
    closeMenu();
    render();
  });

  cancelBtn.addEventListener("click", () => {
    closeUnitSelect();
    render();
  });

  openFormationBtn.addEventListener("click", () => {
    closeUnitSelect();
    showFormationScreen();
    render();
  });

  cancelEmptyBtn.addEventListener("click", () => {
    closeUnitSelect();
    render();
  });

  unitSelectEl.addEventListener("click", (e) => {
    if (e.target === unitSelectEl) {
      closeUnitSelect();
      render();
    }
  });
}

function bindUnitSelectEscape(): void {
  unitSelectEscapeHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      closeUnitSelect();
      render();
    }
  };
  document.addEventListener("keydown", unitSelectEscapeHandler);
}

function unbindUnitSelectEscape(): void {
  if (unitSelectEscapeHandler) {
    document.removeEventListener("keydown", unitSelectEscapeHandler);
    unitSelectEscapeHandler = null;
  }
}
