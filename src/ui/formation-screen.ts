/**
 * 編成画面 — エントリポイント
 * 画面全体を構成し、各ビュー間のイベント設定を統合
 */

import { render, getNextFormedUnitId, formedUnitsList, gameState } from "../store";
import { getUnitCapacity } from "../game/facility-selectors";
import { validateFormedUnits } from "../game/formation";
import { appendFormedUnit } from "../store-actions";
import {
  createFormationElement,
  showFormationHub,
  showFormationScreen,
  closeFormationScreen,
  showFormationHubPanel,
  showFormationModalPanel,
  renderFormationContent,
  clearFormationSlot,
} from "./formation-hub-view";
import {
  openMonsterBrowse,
  setupCharacterPickerListeners,
  openCharacterPicker,
  renderMonsterBrowseGrid,
  renderCharacterPicker,
} from "./formation-monster-view";
import {
  openCardDetailBodySlot,
  showCardDetail,
  refreshCardDetailContent,
  findLatestProduceFeedback,
  setProduceError,
  setupCardDetailOverlayListener,
  getPendingProduceBodySlot,
  clearPendingProduceBodySlot,
} from "./formation-card-view";
import {
  statAllocBodySlot,
  setupStatAllocOverlayListener,
} from "./formation-stat-dialog";
import { shared } from "./formation-shared";

// Re-exports for external consumers
export { showFormationHub, showFormationScreen, closeFormationScreen };
export { openCardDetailBodySlot, statAllocBodySlot };
export { createFormationElement };

export function createFormationElementAndSetup(): HTMLDivElement {
  const el = createFormationElement();
  wireUpFormationScreen();
  return el;
}

/** サーバー state 更新後、編成画面を開いたままなら表示を同期 */
export function refreshFormationScreenIfOpen(): void {
  if (!shared.formationEl?.classList.contains("is-open")) return;
  if (statAllocBodySlot !== null) return;

  validateFormedUnits();
  if (shared.formationView === "units") {
    renderFormationContent();
  } else if (shared.formationView === "monsters") {
    renderMonsterBrowseGrid();
  }
  if (shared.characterPickerEl?.classList.contains("is-open") && shared.charPickerMode === "assign") {
    renderCharacterPicker();
  }
  if (openCardDetailBodySlot !== null && shared.formationEl) {
    refreshCardDetailContent(shared.formationEl, openCardDetailBodySlot);
    if (getPendingProduceBodySlot() === openCardDetailBodySlot) {
      const overlay = shared.formationEl.querySelector(".formation-card-detail-content");
      const feedback = findLatestProduceFeedback(openCardDetailBodySlot);
      if (overlay && feedback) {
        setProduceError(overlay as HTMLElement, feedback.includes("生産した") ? null : feedback);
      }
      clearPendingProduceBodySlot();
    }
  }
}

function getMaxUnits(): number {
  return Math.max(1, 1 + getUnitCapacity(gameState));
}

function wireUpFormationScreen(): void {
  // Hub buttons
  shared.formationEl?.querySelector("[data-formation-hub='monsters']")?.addEventListener("click", () => { openMonsterBrowse(); });
  shared.formationEl?.querySelector("[data-formation-hub='units']")?.addEventListener("click", () => {
    showFormationModalPanel();
    renderFormationContent();
    (shared.formationEl?.querySelector("[data-formation-close]") as HTMLElement)?.focus();
  });
  shared.formationEl?.querySelector("[data-formation-hub-close]")?.addEventListener("click", () => { closeFormationScreen(); render(); });
  shared.formationEl?.querySelector("[data-formation-close]")?.addEventListener("click", () => {
    if (shared.formationView === "units") { showFormationHubPanel(); return; }
    closeFormationScreen(); render();
  });
  shared.formationEl?.querySelector("[data-formation-add-unit]")?.addEventListener("click", () => {
    if (formedUnitsList.length >= getMaxUnits()) return;
    const id = `unit-${getNextFormedUnitId()}`;
    const name = `ユニット${formedUnitsList.length + 1}`;
    appendFormedUnit({ id, name, indices: [-1, -1, -1], monster_count: 0, avgSpeed: 0 });
    renderFormationContent();
  });

  // Unit list interactions (long press + click)
  const listEl = shared.formationEl?.querySelector("[data-formation-unit-list]")!;
  let longPressTimer: number | null = null;
  let didLongPress = false;
  const LONG_PRESS_MS = 500;

  listEl.addEventListener("pointerdown", (e) => {
    didLongPress = false;
    if ((e.target as HTMLElement).closest(".formation-slot-unassign")) return;
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (!slotBtn || slotBtn.classList.contains("formation-slot-empty")) return;
    const charIndex = slotBtn.dataset.charIndex;
    if (!charIndex) return;
    slotBtn.setPointerCapture((e as PointerEvent).pointerId);
    longPressTimer = window.setTimeout(() => {
      longPressTimer = null; didLongPress = true;
      showCardDetail(shared.formationEl!, parseInt(charIndex, 10));
    }, LONG_PRESS_MS);
  });
  listEl.addEventListener("pointerup", (e) => {
    if (longPressTimer !== null) { clearTimeout(longPressTimer); longPressTimer = null; }
    if (didLongPress) { e.preventDefault(); e.stopPropagation(); didLongPress = false; }
  });
  listEl.addEventListener("pointercancel", () => { if (longPressTimer !== null) { clearTimeout(longPressTimer); longPressTimer = null; } });
  listEl.addEventListener("pointerleave", () => { if (longPressTimer !== null) { clearTimeout(longPressTimer); longPressTimer = null; } });

  listEl.addEventListener("click", (e) => {
    const unassignBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot-unassign");
    if (unassignBtn) {
      e.stopPropagation();
      const uid = unassignBtn.dataset.unitId;
      const sidx = parseInt(unassignBtn.dataset.slotIndex ?? "-1", 10);
      if (uid && sidx >= 0 && sidx <= 2) clearFormationSlot(uid, sidx as 0 | 1 | 2);
      return;
    }
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (slotBtn) {
      if (longPressTimer !== null) { clearTimeout(longPressTimer); longPressTimer = null; }
      if (didLongPress) { didLongPress = false; return; }
      const unitId = slotBtn.dataset.unitId;
      const slotIndex = parseInt(slotBtn.dataset.slotIndex ?? "-1", 10);
      if (unitId && slotIndex >= 0 && slotIndex <= 2) openCharacterPicker(unitId, slotIndex as 0 | 1 | 2);
    }
  });

  setupCardDetailOverlayListener(shared.formationEl!);
  setupStatAllocOverlayListener(shared.formationEl!);

  // Character picker listeners
  const gridEl = shared.characterPickerEl?.querySelector("[data-char-picker-grid]") as HTMLDivElement;
  setupCharacterPickerListeners(gridEl, shared.characterPickerEl!);

  // Click outside to dismiss
  shared.formationEl?.addEventListener("click", (e) => {
    if (e.target !== shared.formationEl) return;
    if (shared.formationView === "hub") { closeFormationScreen(); render(); }
    else if (shared.formationView === "units") { showFormationHubPanel(); }
  });
}
