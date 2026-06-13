/**
 * ユニット選択オーバーレイ
 */

import type { PendingUnitAction } from "../store";
import {
  ws,
  formedUnitsList,
  bodyMonsterCounts,
  pendingUnitAction, setPendingUnitAction,
  render,
  getLocalPlayerId,
} from "../store";
import { DEFAULT_BODY_MONSTER_COUNT, getBodyDisplayName, getCharacterSkillData } from "../game/characters";
import { cardStatsToPayload, getEffectiveCardStats } from "../game/effective-stats";
import {
  formationBodyIndicesInSlotOrder,
  isKcUnitReadyToDeploy,
} from "../game/formation";
import { gameState } from "../store";
import { getPlayerOwnedCards, startMarchAction, getPlayerMarches, explorationMaxSlots, activeExploreBodiesInFlight } from "../shared/game-state";
import type { MarchKind } from "../shared/game-state";
import { getPlayerHomeTerritoryId } from "../game/territories";
import { closeMenu } from "./context-menu";
import { showFormationScreen } from "./formation-screen";
import {
  getUnitSelectSnapshot,
  renderAvailableUnits,
  renderReturningUnits,
} from "../unit-select-view";

let unitSelectEl: HTMLDivElement;
let unitSelectEscapeHandler: ((e: KeyboardEvent) => void) | null = null;

function setUnitSelectError(message: string | null): void {
  const el = unitSelectEl.querySelector<HTMLParagraphElement>("[data-unit-error]");
  if (!el) return;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

export function createUnitSelectElement(): HTMLDivElement {
  unitSelectEl = document.createElement("div");
  unitSelectEl.className = "unit-select-overlay";
  unitSelectEl.innerHTML = `
    <div class="unit-select-modal">
      <div class="unit-select-title" data-unit-title>ユニットを選択</div>
      <p class="unit-select-error" data-unit-error hidden></p>
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
        <p class="unit-select-empty-msg">編成されたユニットがありません。編成画面でリーダー枠にキャラを置いてユニットを編成してください。</p>
        <div class="unit-select-returning unit-select-returning--empty" data-unit-returning-empty style="display:none">
          <div class="unit-select-returning-title">帰還中</div>
          <div class="unit-select-returning-list" data-unit-returning-list-empty></div>
        </div>
        <div class="unit-select-actions">
          <button type="button" class="secondary" data-unit-cancel-empty>キャンセル</button>
          <button type="button" class="primary" data-unit-open-formation>編成画面を開く</button>
        </div>
      </div>
    </div>
  `;
  setupUnitSelect();
  return unitSelectEl;
}

export function showUnitSelect(pending: PendingUnitAction): void {
  if (!pending) return;
  setPendingUnitAction(pending);
  setUnitSelectError(null);
  const titleEl = unitSelectEl.querySelector("[data-unit-title]")!;
  const panelForm = unitSelectEl.querySelector("[data-unit-panel-form]") as HTMLElement;
  const panelEmpty = unitSelectEl.querySelector("[data-unit-panel-empty]") as HTMLElement;
  const troopsEl = unitSelectEl.querySelector("[data-unit-troops]")!;
  const listEl = unitSelectEl.querySelector("[data-unit-list]")!;

  if (pending.type === "attack") {
    titleEl.textContent = "攻撃するユニットを選択";
  } else if (pending.type === "explore") {
    titleEl.textContent = "探索するユニットを選択";
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
        : "編成されたユニットがありません。編成画面でリーダー枠にキャラを置いてユニットを編成してください。";
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

/** 体スロット番号は `owned_cards` の添字と同一（サーバー・編成の前提） */
function resolveOwnedCardIndices(bodyIndices: number[], owned: number[]): number[] | null {
  if (owned.length === 0) return null;
  for (const bi of bodyIndices) {
    if (bi < 0 || bi >= owned.length) return null;
  }
  return [...bodyIndices];
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
        : "編成されたユニットがありません。編成画面でリーダー枠にキャラを置いてユニットを編成してください。";
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
    if (!pending) {
      closeUnitSelect();
      render();
      return;
    }
    if (ws?.readyState !== WebSocket.OPEN) {
      setUnitSelectError("サーバーに接続されていません。接続後にもう一度決定してください。");
      return;
    }
    const listEl = unitSelectEl.querySelector("[data-unit-list]")!;
    const checked = listEl.querySelector<HTMLInputElement>('input[name="unit-select-one"]:checked');
    const selectedId = checked?.dataset.unitId ?? null;
    if (!selectedId) {
      setUnitSelectError("送信するユニットを一覧から選んでください。");
      return;
    }
    const unit = formedUnitsList.find((u) => u.id === selectedId);
    if (!unit) {
      setUnitSelectError("選択したユニットが見つかりません。一覧を更新してからもう一度選んでください。");
      return;
    }
    if (!isKcUnitReadyToDeploy(unit.indices)) {
      setUnitSelectError("リーダー枠にキャラがいないユニットは出撃できません。編成画面でリーダーを配置してください。");
      return;
    }
    const bodyIdxOrder = formationBodyIndicesInSlotOrder(unit.indices);
    let dispatchIndices = bodyIdxOrder;
    if (pending.type === "explore") {
      const playerId = getLocalPlayerId();
      const exploreLevel = gameState.players[playerId]?.exploration_level ?? 1;
      const maxSlots = explorationMaxSlots(exploreLevel);
      const activeBodies = activeExploreBodiesInFlight(getPlayerMarches(gameState, playerId));
      const slotsRemaining = maxSlots - activeBodies;
      if (slotsRemaining <= 0) {
        setUnitSelectError("これ以上探索を出せません。進行中の探索が終わるまでお待ちください。");
        return;
      }
      if (bodyIdxOrder.length > slotsRemaining) {
        dispatchIndices = bodyIdxOrder.slice(0, slotsRemaining);
      }
    }
    const count = dispatchIndices.length;
    if (count === 0) {
      setUnitSelectError("このユニットに出撃する体がありません。");
      return;
    }
    const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
    const monstersPerBody = dispatchIndices.map((i) => bodyMonsterCounts[i] ?? DEFAULT_BODY_MONSTER_COUNT);
    const speedPerBody = dispatchIndices.map((i) => {
      const cid = owned[i] ?? 0;
      return getEffectiveCardStats(cid, i, gameState, getLocalPlayerId()).speed;
    });
    const bodyNames = dispatchIndices.map((i) => {
      const cid = owned[i] ?? 0;
      return getBodyDisplayName(cid);
    });
    const skillsPerBody = dispatchIndices.map((i) => {
      const cid = owned[i] ?? 0;
      return getCharacterSkillData(cid);
    });
    const statsPerBody = dispatchIndices.map((i) => {
      const cid = owned[i] ?? 0;
      const s = getEffectiveCardStats(cid, i, gameState, getLocalPlayerId());
      const mc = bodyMonsterCounts[i] ?? s.monster_count;
      return { ...cardStatsToPayload(s), monster_count: mc };
    });
    const ownedCardIndices = resolveOwnedCardIndices(dispatchIndices, owned);
    if (!ownedCardIndices) {
      setUnitSelectError(
        "このユニットの体番号と、本拠の所持魔獣スロットが一致しません。編成の体番号をスロット内に収めるか、編成し直してください。",
      );
      return;
    }

    let kind: MarchKind;
    let fromId: string;
    let toId: string;
    let formedUnitId: string | undefined;

    if (pending.type === "attack") {
      kind = "attack";
      fromId = pending.fromId;
      toId = pending.toId;
      formedUnitId = unit.id;
    } else if (pending.type === "explore") {
      kind = "explore";
      fromId = pending.fromId;
      toId = pending.toId;
      // 探索は同時派遣数に応じて体を分けて送る（編成ユニット全体のロックはしない）
      formedUnitId = undefined;
    } else {
      kind = "deploy";
      fromId = getPlayerHomeTerritoryId(gameState, getLocalPlayerId());
      toId = pending.territoryId;
      formedUnitId = unit.id;
    }

    ws.send(JSON.stringify(startMarchAction(kind, fromId, toId, count, {
      monstersPerBody,
      bodyNames,
      unitName: unit.name,
      speedPerBody,
      skillsPerBody,
      statsPerBody,
      ownedCardIndices,
      formedUnitId,
    })));

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
