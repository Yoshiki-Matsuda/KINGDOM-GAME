/**
 * 編成画面UI
 * ユニット一覧 → 行クリックで3枠表示 → 枠クリックでキャラ選択（イラスト羅列）
 */

import {
  bodyMonsterCounts, bodySpeeds,
  formedUnitsList, setFormedUnitsList,
  getNextFormedUnitId,
  gameState, render, ws,
} from "../store";
import { getPlayerOwnedCards, produceMonstersAction } from "../shared/game-state";
import { getBodyDisplayName, getCharacterIllustrationPath, getCharacterStats } from "../game/characters";
import { getHomeTroops, validateFormedUnits, recalcUnitStats } from "../game/formation";
import { getEffectiveUnitCostCap, getUnitCapacity } from "../game/facility-selectors";
import { buildFormationCardDetailHtml } from "../formation-card-detail";
import { appendFormedUnit } from "../store-actions";
import { escapeHtml } from "../utils";

let formationEl: HTMLDivElement;
let characterPickerEl: HTMLDivElement;

/** 編集中: どのユニットのどのスロットか */
let editingUnitId: string | null = null;
let editingSlotIndex: 0 | 1 | 2 | null = null;

/** ユニット上限（施設ボーナス込み） */
function getMaxUnits(): number {
  return Math.max(1, 1 + getUnitCapacity(gameState));
}

function unitFilledCostSum(indices: [number, number, number]): number {
  const owned = getPlayerOwnedCards(gameState);
  let s = 0;
  for (const i of indices) {
    if (i >= 0) s += getCharacterStats(owned[i] ?? 0).cost;
  }
  return s;
}

function clearFormationSlot(unitId: string, slotIndex: 0 | 1 | 2): void {
  const unit = formedUnitsList.find((u) => u.id === unitId);
  if (!unit || unit.indices[slotIndex] < 0) return;
  const newIndices: [number, number, number] = [...unit.indices];
  newIndices[slotIndex] = -1;
  const { monster_count, avgSpeed } = recalcUnitStats(newIndices, bodyMonsterCounts, bodySpeeds);
  const idx = formedUnitsList.findIndex((u) => u.id === unitId);
  const updated = [...formedUnitsList];
  updated[idx] = { ...unit, indices: newIndices, monster_count, avgSpeed };
  setFormedUnitsList(updated);
  renderFormationContent();
}

export function createFormationElement(): HTMLDivElement {
  formationEl = document.createElement("div");
  formationEl.className = "formation-overlay";
  formationEl.innerHTML = `
    <div class="formation-modal">
      <div class="formation-title">ユニット編成</div>
      <div class="formation-desc">枠をクリックしてキャラを選択。「外す」で枠を空に戻せます</div>
      <div class="formation-troops" data-formation-troops>本拠地: 0 体</div>
      <div class="formation-unit-list" data-formation-unit-list></div>
      <button type="button" class="formation-add-unit" data-formation-add-unit>新規ユニットを追加</button>
      <button type="button" class="formation-close" data-formation-close>編成画面を閉じる</button>
    </div>
  `;

  characterPickerEl = document.createElement("div");
  characterPickerEl.className = "formation-char-picker-overlay";
  characterPickerEl.innerHTML = `
    <div class="formation-char-picker-modal">
      <div class="formation-char-picker-title">魔獣一覧</div>
      <div class="formation-char-picker-grid" data-char-picker-grid></div>
      <button type="button" class="formation-char-picker-close" data-char-picker-close>閉じる</button>
    </div>
  `;
  formationEl.appendChild(characterPickerEl);

  const cardDetailEl = document.createElement("div");
  cardDetailEl.className = "formation-card-detail-overlay";
  cardDetailEl.innerHTML = `
    <div class="formation-card-detail-modal">
      <div class="formation-card-detail-content" data-card-detail-content></div>
      <button type="button" class="formation-card-detail-close" data-card-detail-close>閉じる</button>
    </div>
  `;
  formationEl.appendChild(cardDetailEl);

  setupFormationScreen();
  return formationEl;
}

export function showFormationScreen(): void {
  validateFormedUnits();
  editingUnitId = null;
  editingSlotIndex = null;
  formationEl.classList.add("is-open");
  characterPickerEl.classList.remove("is-open");
  const mapContainer = document.querySelector<HTMLElement>(".map-container");
  if (mapContainer) mapContainer.style.pointerEvents = "none";
  renderFormationContent();
  (formationEl.querySelector("[data-formation-close]") as HTMLElement)?.focus();
}

export function closeFormationScreen(): void {
  formationEl.classList.remove("is-open");
  characterPickerEl.classList.remove("is-open");
  closeCardDetail();
  const mapContainer = document.querySelector<HTMLElement>(".map-container");
  if (mapContainer) mapContainer.style.pointerEvents = "";
}

function openCharacterPicker(unitId: string, slotIndex: 0 | 1 | 2): void {
  editingUnitId = unitId;
  editingSlotIndex = slotIndex;
  renderCharacterPicker();
  characterPickerEl.classList.add("is-open");
}

function closeCharacterPicker(): void {
  editingUnitId = null;
  editingSlotIndex = null;
  characterPickerEl.classList.remove("is-open");
}

let cardDetailOpenedAt = 0;
const CARD_DETAIL_CLOSE_DELAY_MS = 800;

function wireProducePanel(contentEl: HTMLElement, bodySlot: number): void {
  const panel = contentEl.querySelector<HTMLElement>("[data-produce-panel]");
  const startBtn = contentEl.querySelector<HTMLButtonElement>("[data-produce-start]");
  const input = contentEl.querySelector<HTMLInputElement>("[data-produce-amount]");
  if (!panel || !startBtn || !input) return;

  const openPanel = (): void => {
    panel.hidden = false;
    startBtn.hidden = true;
    input.value = "1";
    input.focus();
    input.select();
  };

  const closePanel = (): void => {
    panel.hidden = true;
    startBtn.hidden = false;
  };

  const sendProduce = (): void => {
    if (ws?.readyState !== WebSocket.OPEN) {
      alert("サーバーに接続されていません。接続後にもう一度お試しください。");
      return;
    }
    const maxAttr = parseInt(input.getAttribute("max") ?? "0", 10);
    const maxAllowed = Number.isFinite(maxAttr) && maxAttr > 0 ? maxAttr : 0;
    const raw = input.value.trim();
    let n = parseInt(raw, 10);
    if (!Number.isFinite(n) || n < 1) {
      alert("生産する数に1以上の整数を入力してください。");
      return;
    }
    if (maxAllowed > 0 && n > maxAllowed) n = maxAllowed;
    ws.send(JSON.stringify(produceMonstersAction(bodySlot, n)));
  };

  startBtn.addEventListener("click", openPanel);
  contentEl.querySelector<HTMLButtonElement>("[data-produce-cancel]")?.addEventListener("click", closePanel);
  contentEl.querySelector<HTMLButtonElement>("[data-produce-submit]")?.addEventListener("click", sendProduce);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      sendProduce();
    }
  });
}

function showCardDetail(bodySlot: number): void {
  const cardDetailEl = formationEl.querySelector(".formation-card-detail-overlay")!;
  const contentEl = cardDetailEl.querySelector("[data-card-detail-content]")! as HTMLElement;
  contentEl.innerHTML = buildFormationCardDetailHtml(bodySlot);
  wireProducePanel(contentEl, bodySlot);
  cardDetailEl.classList.add("is-open");
  cardDetailOpenedAt = Date.now();
}

function closeCardDetail(): void {
  formationEl.querySelector(".formation-card-detail-overlay")?.classList.remove("is-open");
}

function renderCharacterPicker(): void {
  if (!editingUnitId || editingSlotIndex === null) return;
  const unit = formedUnitsList.find((u) => u.id === editingUnitId);
  if (!unit) return;

  const homeTroops = getHomeTroops();
  // 他ユニットで使用中のキャラ（編集中ユニットのメンバーは除外＝入れ替え可能）
  const usedByOthers = new Set(
    formedUnitsList
      .filter((u) => u.id !== editingUnitId)
      .flatMap((u) => u.indices)
      .filter((i) => i >= 0)
  );

  const gridEl = characterPickerEl.querySelector("[data-char-picker-grid]")!;
  gridEl.innerHTML = "";
  const ownedCards = getPlayerOwnedCards(gameState);

  for (let i = 0; i < homeTroops; i++) {
    const used = usedByOthers.has(i);
    const isCurrentSlot = unit.indices[editingSlotIndex] === i;
    const canSelect = !used || isCurrentSlot;

    const card = document.createElement("button");
    card.type = "button";
    card.className = "formation-char-picker-card" + (isCurrentSlot ? " is-selected" : "") + (used && !isCurrentSlot ? " is-used" : "");
    card.dataset.charIndex = String(i);
    if (!canSelect) card.disabled = true;

    const cardId = ownedCards[i] ?? i;
    const img = document.createElement("img");
    img.src = getCharacterIllustrationPath(cardId);
    img.alt = getBodyDisplayName(cardId);
    img.className = "formation-char-picker-img";
    card.appendChild(img);
    const nameEl = document.createElement("div");
    nameEl.className = "formation-char-picker-name";
    nameEl.textContent = getBodyDisplayName(cardId);
    card.appendChild(nameEl);
    gridEl.appendChild(card);
  }
}

function renderFormationContent(): void {
  const homeTroops = getHomeTroops();
  const maxUnits = getMaxUnits();
  const costCap = getEffectiveUnitCostCap(gameState);
  const troopsEl = formationEl.querySelector("[data-formation-troops]")!;
  const listEl = formationEl.querySelector("[data-formation-unit-list]")!;
  const addBtn = formationEl.querySelector<HTMLButtonElement>("[data-formation-add-unit]")!;

  troopsEl.textContent = `本拠地: ${homeTroops} 体・コスト上限 ${costCap.toFixed(1)}`;
  addBtn.disabled = formedUnitsList.length >= maxUnits;

  listEl.innerHTML = "";
  const ownedCards = getPlayerOwnedCards(gameState);
  formedUnitsList.forEach((u) => {
    const row = document.createElement("div");
    row.className = "formation-unit-row";
    row.dataset.unitId = u.id;

    const header = document.createElement("div");
    header.className = "formation-unit-row-header";
    const filledCost = unitFilledCostSum(u.indices);
    const costHint = u.indices.some((i) => i >= 0)
      ? ` <span class="formation-unit-cost">コスト ${filledCost.toFixed(1)} / ${costCap.toFixed(1)}</span>`
      : "";
    header.innerHTML = `<span class="formation-unit-row-name">${escapeHtml(u.name)}</span>${costHint}`;
    row.appendChild(header);

    const slots = document.createElement("div");
    slots.className = "formation-unit-slots";
    const POSITION_LABELS = ["FRONT", "BACK", "LEADER"] as const;
    const POSITION_RANGE_HINT = ["射程1+", "射程2+", "射程3で狙われる"] as const;
    for (let s = 0; s < 3; s++) {
      const wrap = document.createElement("div");
      wrap.className = "formation-slot-wrap";

      const slot = document.createElement("button");
      slot.type = "button";
      slot.className = "formation-slot";
      slot.dataset.unitId = u.id;
      slot.dataset.slotIndex = String(s);

      const posLabel = document.createElement("div");
      posLabel.className = "formation-slot-position";
      posLabel.textContent = POSITION_LABELS[s];
      slot.appendChild(posLabel);

      const idx = u.indices[s];
      if (idx >= 0) {
        slot.dataset.charIndex = String(idx);
        const cardId = ownedCards[idx] ?? idx;
        const img = document.createElement("img");
        img.src = getCharacterIllustrationPath(cardId);
        img.alt = getBodyDisplayName(cardId);
        slot.appendChild(img);
        const nameSpan = document.createElement("span");
        nameSpan.className = "formation-slot-name";
        nameSpan.textContent = getBodyDisplayName(cardId);
        slot.appendChild(nameSpan);
      } else {
        slot.classList.add("formation-slot-empty");
        const hint = document.createElement("div");
        hint.className = "formation-slot-hint";
        hint.innerHTML = `<span class="formation-slot-plus">+</span><span class="formation-slot-range-hint">${POSITION_RANGE_HINT[s]}</span>`;
        slot.appendChild(hint);
      }
      wrap.appendChild(slot);

      if (idx >= 0) {
        const unassign = document.createElement("button");
        unassign.type = "button";
        unassign.className = "formation-slot-unassign";
        unassign.dataset.unitId = u.id;
        unassign.dataset.slotIndex = String(s);
        unassign.textContent = "外す";
        wrap.appendChild(unassign);
      }

      slots.appendChild(wrap);
    }
    row.appendChild(slots);

    listEl.appendChild(row);
  });
}

function setupFormationScreen(): void {
  formationEl.querySelector("[data-formation-close]")?.addEventListener("click", () => {
    closeFormationScreen();
    render();
  });

  formationEl.querySelector("[data-formation-add-unit]")?.addEventListener("click", () => {
    if (formedUnitsList.length >= getMaxUnits()) return;
    const id = `unit-${getNextFormedUnitId()}`;
    const name = `ユニット${formedUnitsList.length + 1}`;
    appendFormedUnit({
      id,
      name,
      indices: [-1, -1, -1],
      monster_count: 0,
      avgSpeed: 0,
    });
    renderFormationContent();
  });

  const listEl = formationEl.querySelector("[data-formation-unit-list]")!;
  let longPressTimer: number | null = null;
  let didLongPress = false;
  const LONG_PRESS_MS = 500;

  listEl.addEventListener("pointerdown", (e) => {
    if ((e.target as HTMLElement).closest(".formation-slot-unassign")) return;
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (!slotBtn || slotBtn.classList.contains("formation-slot-empty")) return;
    const charIndex = slotBtn.dataset.charIndex;
    if (!charIndex) return;
    didLongPress = false;
    slotBtn.setPointerCapture((e as PointerEvent).pointerId);
    longPressTimer = window.setTimeout(() => {
      longPressTimer = null;
      didLongPress = true;
      showCardDetail(parseInt(charIndex, 10));
    }, LONG_PRESS_MS);
  });

  listEl.addEventListener("pointerup", (e) => {
    if (longPressTimer !== null) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
    if (didLongPress) {
      e.preventDefault();
      e.stopPropagation();
    }
  });
  listEl.addEventListener("pointercancel", () => {
    if (longPressTimer !== null) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
  });
  listEl.addEventListener("pointerleave", () => {
    if (longPressTimer !== null) {
      clearTimeout(longPressTimer);
      longPressTimer = null;
    }
  });

  listEl.addEventListener("click", (e) => {
    const unassignBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot-unassign");
    if (unassignBtn) {
      e.stopPropagation();
      const uid = unassignBtn.dataset.unitId;
      const sidx = parseInt(unassignBtn.dataset.slotIndex ?? "-1", 10);
      if (uid && sidx >= 0 && sidx <= 2) {
        clearFormationSlot(uid, sidx as 0 | 1 | 2);
      }
      return;
    }
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (slotBtn) {
      if (longPressTimer !== null) {
        clearTimeout(longPressTimer);
        longPressTimer = null;
      }
      if (didLongPress) return;
      const unitId = slotBtn.dataset.unitId;
      const slotIndex = parseInt(slotBtn.dataset.slotIndex ?? "-1", 10);
      if (unitId && slotIndex >= 0 && slotIndex <= 2) {
        openCharacterPicker(unitId, slotIndex as 0 | 1 | 2);
      }
      return;
    }
  });

  const cardDetailOverlay = formationEl.querySelector(".formation-card-detail-overlay");
  cardDetailOverlay?.querySelector("[data-card-detail-close]")?.addEventListener("click", closeCardDetail);
  cardDetailOverlay?.addEventListener("click", (e) => {
    if (e.target !== cardDetailOverlay) return;
    if (Date.now() - cardDetailOpenedAt < CARD_DETAIL_CLOSE_DELAY_MS) return;
    closeCardDetail();
  });

  const gridEl = characterPickerEl.querySelector("[data-char-picker-grid]")!;
  let pickerLongPressTimer: number | null = null;
  let pickerDidLongPress = false;

  gridEl.addEventListener("pointerdown", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    const charIndex = card.dataset.charIndex;
    if (!charIndex) return;
    pickerDidLongPress = false;
    card.setPointerCapture((e as PointerEvent).pointerId);
    pickerLongPressTimer = window.setTimeout(() => {
      pickerLongPressTimer = null;
      pickerDidLongPress = true;
      showCardDetail(parseInt(charIndex, 10));
    }, LONG_PRESS_MS);
  });

  gridEl.addEventListener("pointerup", (e) => {
    if (pickerLongPressTimer !== null) {
      clearTimeout(pickerLongPressTimer);
      pickerLongPressTimer = null;
    }
    if (pickerDidLongPress) {
      e.preventDefault();
      e.stopPropagation();
    }
  });
  gridEl.addEventListener("pointercancel", () => {
    if (pickerLongPressTimer !== null) {
      clearTimeout(pickerLongPressTimer);
      pickerLongPressTimer = null;
    }
  });
  gridEl.addEventListener("pointerleave", () => {
    if (pickerLongPressTimer !== null) {
      clearTimeout(pickerLongPressTimer);
      pickerLongPressTimer = null;
    }
  });

  characterPickerEl.querySelector("[data-char-picker-close]")?.addEventListener("click", () => {
    closeCharacterPicker();
  });

  gridEl.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    if (pickerDidLongPress) return;
    const charIndex = parseInt(card.dataset.charIndex ?? "-1", 10);
    if (charIndex < 0 || !editingUnitId || editingSlotIndex === null) return;

    const unit = formedUnitsList.find((u) => u.id === editingUnitId);
    if (!unit) return;

    const newIndices: [number, number, number] = [...unit.indices];
    newIndices[editingSlotIndex] = charIndex;
    const tryCost = unitFilledCostSum(newIndices);
    const cap = getEffectiveUnitCostCap(gameState);
    if (tryCost > cap + 0.0001) {
      alert(
        `ユニットコスト上限（${cap.toFixed(1)}）を超えます（この編成だと合計${tryCost.toFixed(1)}）。別の体を選ぶか、既に入れた枠の「外す」でコストを空けてください。`,
      );
      return;
    }
    const { monster_count, avgSpeed } = recalcUnitStats(newIndices, bodyMonsterCounts, bodySpeeds);

    const idx = formedUnitsList.findIndex((u) => u.id === editingUnitId);
    const updated = [...formedUnitsList];
    updated[idx] = { ...unit, indices: newIndices, monster_count, avgSpeed };
    setFormedUnitsList(updated);

    closeCharacterPicker();
    renderFormationContent();
  });

  characterPickerEl.addEventListener("click", (e) => {
    if (e.target === characterPickerEl) closeCharacterPicker();
  });

  formationEl.addEventListener("click", (e) => {
    if (e.target === formationEl) {
      closeFormationScreen();
      render();
    }
  });
}
