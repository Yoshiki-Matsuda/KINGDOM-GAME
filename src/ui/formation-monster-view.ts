/**
 * 編成画面 — モンスタービュー
 * 魔獣一覧、キャラピッカー、カード詳細表示
 */

import {
  bodyMonsterCounts, bodySpeeds,
  formedUnitsList, getLocalPlayerId,
  gameState,
} from "../store";
import {
  getMarchLockedCardSlots,
  getPlayerOwnedCards,
} from "../shared/game-state";
import {
  getBodyDisplayName, getCardRarityClass, getCharacterIllustrationPath,
  getCharacterStats, getUniqueIllustratedSpeciesSlots,
} from "../game/characters";
import { getEffectiveUnitCostCap } from "../game/facility-selectors";
import { getHomeTroops, recalcUnitStats } from "../game/formation";
import { commitFormedUnits } from "../game/formed-units-persist";
import { showCardDetail } from "./formation-card-view";
import {
  renderFormationContent,
  showFormationHubPanel,
} from "./formation-hub-view";
import { shared } from "./formation-shared";

function setCharPickerError(message: string | null): void {
  const el = shared.characterPickerEl?.querySelector<HTMLParagraphElement>("[data-char-picker-error]");
  if (!el) return;
  if (message) { el.textContent = message; el.hidden = false; }
  else { el.hidden = true; }
}

export function openMonsterBrowse(): void {
  shared.formationView = "monsters";
  shared.charPickerMode = "browse";
  shared.editingUnitId = null;
  shared.editingSlotIndex = null;
  setCharPickerError(null);
  shared.hubEl?.classList.remove("is-active");
  shared.formationModalEl?.classList.remove("is-active");
  updateCharPickerChrome();
  renderMonsterBrowseGrid();
  shared.characterPickerEl?.classList.add("is-open");
}

function updateCharPickerChrome(): void {
  const titleEl = shared.characterPickerEl?.querySelector("[data-char-picker-title]")!;
  const descEl = shared.characterPickerEl?.querySelector<HTMLElement>("[data-char-picker-desc]")!;
  const closeBtn = shared.characterPickerEl?.querySelector<HTMLButtonElement>("[data-char-picker-close]")!;
  if (shared.charPickerMode === "browse") {
    titleEl.textContent = "魔獣一覧";
    descEl.hidden = false;
    closeBtn.textContent = "戻る";
  } else {
    titleEl.textContent = "魔獣を選択";
    descEl.hidden = true;
    closeBtn.textContent = "閉じる";
  }
}

export function openCharacterPicker(unitId: string, slotIndex: 0 | 1 | 2): void {
  shared.editingUnitId = unitId;
  shared.editingSlotIndex = slotIndex;
  shared.charPickerMode = "assign";
  setCharPickerError(null);
  updateCharPickerChrome();
  renderCharacterPicker();
  shared.characterPickerEl?.classList.add("is-open");
}

export function closeCharacterPicker(): void {
  shared.characterPickerEl?.classList.remove("is-open");
  if (shared.charPickerMode === "browse") {
    shared.charPickerMode = "assign";
    showFormationHubPanel();
    return;
  }
  shared.editingUnitId = null;
  shared.editingSlotIndex = null;
}

function unitFilledCostSum(indices: [number, number, number]): number {
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  let s = 0;
  for (const i of indices) {
    if (i >= 0) s += getCharacterStats(owned[i] ?? 0).cost;
  }
  return s;
}

export function renderMonsterBrowseGrid(): void {
  const gridEl = shared.characterPickerEl?.querySelector("[data-char-picker-grid]")!;
  gridEl.innerHTML = "";
  const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const slots = getUniqueIllustratedSpeciesSlots(ownedCards);

  for (const i of slots) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = "formation-char-picker-card";
    card.dataset.charIndex = String(i);

    const cardId = ownedCards[i] ?? i;
    const img = document.createElement("img");
    img.src = getCharacterIllustrationPath(cardId);
    img.alt = getBodyDisplayName(cardId);
    img.className = "formation-char-picker-img";
    card.appendChild(img);
    const nameEl = document.createElement("div");
    nameEl.className = `formation-char-picker-name ${getCardRarityClass(cardId)}`;
    nameEl.textContent = getBodyDisplayName(cardId);
    card.appendChild(nameEl);
    gridEl.appendChild(card);
  }
}

export function renderCharacterPicker(): void {
  if (!shared.editingUnitId || shared.editingSlotIndex === null) return;
  const unit = formedUnitsList.find((u) => u.id === shared.editingUnitId);
  if (!unit) return;

  const homeTroops = getHomeTroops();
  const usedByOthers = new Set(
    formedUnitsList.filter((u) => u.id !== shared.editingUnitId).flatMap((u) => u.indices).filter((i) => i >= 0)
  );

  const gridEl = shared.characterPickerEl?.querySelector("[data-char-picker-grid]")!;
  gridEl.innerHTML = "";
  const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const marchLockedSlots = getMarchLockedCardSlots(gameState, getLocalPlayerId());
  const cardIdsInUnit = new Set(
    unit.indices.filter((idx, slot) => idx >= 0 && slot !== shared.editingSlotIndex).map((idx) => ownedCards[idx] ?? idx),
  );

  for (let i = 0; i < homeTroops; i++) {
    const used = usedByOthers.has(i);
    const isCurrentSlot = unit.indices[shared.editingSlotIndex] === i;
    const sameSpeciesInUnit = cardIdsInUnit.has(ownedCards[i] ?? i);
    const onMarch = marchLockedSlots.has(i);
    const canSelect = (!used || isCurrentSlot) && (!sameSpeciesInUnit || isCurrentSlot) && (!onMarch || isCurrentSlot);

    const card = document.createElement("button");
    card.type = "button";
    card.className =
      "formation-char-picker-card" +
      (isCurrentSlot ? " is-selected" : "") +
      ((used && !isCurrentSlot) || (onMarch && !isCurrentSlot) ? " is-used" : "");
    card.dataset.charIndex = String(i);
    if (!canSelect) card.disabled = true;

    const cardId = ownedCards[i] ?? i;
    const img = document.createElement("img");
    img.src = getCharacterIllustrationPath(cardId);
    img.alt = getBodyDisplayName(cardId);
    img.className = "formation-char-picker-img";
    card.appendChild(img);
    const nameEl = document.createElement("div");
    nameEl.className = `formation-char-picker-name ${getCardRarityClass(cardId)}`;
    nameEl.textContent = getBodyDisplayName(cardId);
    card.appendChild(nameEl);
    gridEl.appendChild(card);
  }
}

/** formationEl に対して char-picker-grid のイベントリスナーを登録 */
export function setupCharacterPickerListeners(
  gridEl: HTMLDivElement,
  pickerEl: HTMLDivElement,
): void {
  let pickerLongPressTimer: number | null = null;
  let pickerDidLongPress = false;
  const LONG_PRESS_MS = 500;

  gridEl.addEventListener("pointerdown", (e) => {
    pickerDidLongPress = false;
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    const charIndex = card.dataset.charIndex;
    if (!charIndex) return;
    card.setPointerCapture((e as PointerEvent).pointerId);
    pickerLongPressTimer = window.setTimeout(() => {
      pickerLongPressTimer = null; pickerDidLongPress = true;
      showCardDetail(shared.formationEl!, parseInt(charIndex, 10));
    }, LONG_PRESS_MS);
  });
  gridEl.addEventListener("pointerup", (e) => {
    if (pickerLongPressTimer !== null) { clearTimeout(pickerLongPressTimer); pickerLongPressTimer = null; }
    if (pickerDidLongPress) { e.preventDefault(); e.stopPropagation(); pickerDidLongPress = false; }
  });
  gridEl.addEventListener("pointercancel", () => { if (pickerLongPressTimer !== null) { clearTimeout(pickerLongPressTimer); pickerLongPressTimer = null; } });
  gridEl.addEventListener("pointerleave", () => { if (pickerLongPressTimer !== null) { clearTimeout(pickerLongPressTimer); pickerLongPressTimer = null; } });

  pickerEl.querySelector("[data-char-picker-close]")?.addEventListener("click", () => { closeCharacterPicker(); });

  gridEl.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    if (pickerDidLongPress) { pickerDidLongPress = false; return; }
    const charIndex = parseInt(card.dataset.charIndex ?? "-1", 10);
    if (charIndex < 0) return;

    if (shared.charPickerMode === "browse") { showCardDetail(shared.formationEl!, charIndex); return; }
    if (!shared.editingUnitId || shared.editingSlotIndex === null) return;

    const unit = formedUnitsList.find((u) => u.id === shared.editingUnitId);
    if (!unit) return;

    const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
    const pickedCardId = ownedCards[charIndex] ?? charIndex;
    const duplicateSpecies = unit.indices.some(
      (idx, slot) => slot !== shared.editingSlotIndex && idx >= 0 && (ownedCards[idx] ?? idx) === pickedCardId,
    );
    if (duplicateSpecies) {
      setCharPickerError("同じ種類の魔獣は1ユニットに1体まです。別の魔獣を選んでください。");
      return;
    }

    const newIndices: [number, number, number] = [...unit.indices];
    newIndices[shared.editingSlotIndex] = charIndex;
    const tryCost = unitFilledCostSum(newIndices);
    const cap = getEffectiveUnitCostCap(gameState);
    if (tryCost > cap + 0.0001) {
      setCharPickerError(
        `ユニットコスト上限（${cap.toFixed(1)}）を超えます（この編成だと合計${tryCost.toFixed(1)}）。別の体を選ぶか、既に入れた枠の「外す」でコストを空けてください。`,
      );
      return;
    }
    setCharPickerError(null);
    const { monster_count, avgSpeed } = recalcUnitStats(newIndices, bodyMonsterCounts, bodySpeeds);
    const idx = formedUnitsList.findIndex((u) => u.id === shared.editingUnitId);
    const updated = [...formedUnitsList];
    updated[idx] = { ...unit, indices: newIndices, monster_count, avgSpeed };
    commitFormedUnits(updated);
    closeCharacterPicker();
    renderFormationContent();
  });

  pickerEl.addEventListener("click", (e) => { if (e.target === pickerEl) closeCharacterPicker(); });
}
