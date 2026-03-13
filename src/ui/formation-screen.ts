/**
 * 編成画面UI
 * ユニット一覧 → 行クリックで3枠表示 → 枠クリックでキャラ選択（イラスト羅列）
 */

import {
  bodyEnergies, bodySpeeds,
  formedUnitsList, setFormedUnitsList,
  getNextFormedUnitId,
  render,
  gameState,
} from "../store";
import { getBodyDisplayName, getCharacterIllustrationPath } from "../game/characters";
import { getHomeTroops, validateFormedUnits, recalcUnitStats } from "../game/formation";
import { calculateFacilityBonuses } from "../game/facilities";
import type { FacilityId } from "../game/facilities";
import { escapeHtml } from "../utils";

let formationEl: HTMLDivElement;
let characterPickerEl: HTMLDivElement;

/** 編集中: どのユニットのどのスロットか */
let editingUnitId: string | null = null;
let editingSlotIndex: 0 | 1 | 2 | null = null;


/** 建設済み施設を取得 */
function getBuiltFacilitiesMap(): Map<FacilityId, number> {
  const built = new Map<FacilityId, number>();
  for (const f of gameState.facilities ?? []) {
    if (!f.build_complete_at || f.build_complete_at <= Date.now()) {
      built.set(f.facility_id as FacilityId, f.level);
    }
  }
  return built;
}

/** ユニット上限（施設ボーナス込み） */
function getMaxUnits(): number {
  const bonuses = calculateFacilityBonuses(getBuiltFacilitiesMap());
  return Math.max(1, 1 + bonuses.unitCapacity);
}

export function createFormationElement(): HTMLDivElement {
  formationEl = document.createElement("div");
  formationEl.className = "formation-overlay";
  formationEl.innerHTML = `
    <div class="formation-modal">
      <div class="formation-title">編成画面</div>
      <div class="formation-desc">枠をクリックしてキャラを選択します</div>
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
      <div class="formation-char-picker-title">キャラを選択</div>
      <div class="formation-char-picker-grid" data-char-picker-grid></div>
      <button type="button" class="formation-char-picker-close" data-char-picker-close>閉じる</button>
    </div>
  `;
  formationEl.appendChild(characterPickerEl);

  setupFormationScreen();
  return formationEl;
}

export function showFormationScreen(): void {
  validateFormedUnits();
  editingUnitId = null;
  editingSlotIndex = null;
  formationEl.classList.add("is-open");
  characterPickerEl.classList.remove("is-open");
  renderFormationContent();
  (formationEl.querySelector("[data-formation-close]") as HTMLElement)?.focus();
}

export function closeFormationScreen(): void {
  formationEl.classList.remove("is-open");
  characterPickerEl.classList.remove("is-open");
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

  for (let i = 0; i < homeTroops; i++) {
    const used = usedByOthers.has(i);
    const isCurrentSlot = unit.indices[editingSlotIndex] === i;
    const canSelect = !used || isCurrentSlot;

    const card = document.createElement("button");
    card.type = "button";
    card.className = "formation-char-picker-card" + (isCurrentSlot ? " is-selected" : "") + (used && !isCurrentSlot ? " is-used" : "");
    card.dataset.charIndex = String(i);
    if (!canSelect) card.disabled = true;

    const img = document.createElement("img");
    img.src = getCharacterIllustrationPath(i);
    img.alt = getBodyDisplayName(i);
    img.className = "formation-char-picker-img";
    card.appendChild(img);
    const nameEl = document.createElement("div");
    nameEl.className = "formation-char-picker-name";
    nameEl.textContent = getBodyDisplayName(i);
    card.appendChild(nameEl);
    gridEl.appendChild(card);
  }
}

function renderFormationContent(): void {
  const homeTroops = getHomeTroops();
  const maxUnits = getMaxUnits();
  const troopsEl = formationEl.querySelector("[data-formation-troops]")!;
  const listEl = formationEl.querySelector("[data-formation-unit-list]")!;
  const addBtn = formationEl.querySelector<HTMLButtonElement>("[data-formation-add-unit]")!;

  troopsEl.textContent = `本拠地: ${homeTroops} 体`;
  addBtn.disabled = formedUnitsList.length >= maxUnits;

  listEl.innerHTML = "";
  formedUnitsList.forEach((u) => {
    const row = document.createElement("div");
    row.className = "formation-unit-row";
    row.dataset.unitId = u.id;

    const header = document.createElement("div");
    header.className = "formation-unit-row-header";
    header.innerHTML = `<span class="formation-unit-row-name">${escapeHtml(u.name)}</span>`;
    row.appendChild(header);

    const slots = document.createElement("div");
    slots.className = "formation-unit-slots";
    for (let s = 0; s < 3; s++) {
      const slot = document.createElement("button");
      slot.type = "button";
      slot.className = "formation-slot";
      slot.dataset.unitId = u.id;
      slot.dataset.slotIndex = String(s);
      const idx = u.indices[s];
      if (idx >= 0) {
        const img = document.createElement("img");
        img.src = getCharacterIllustrationPath(idx);
        img.alt = getBodyDisplayName(idx);
        slot.appendChild(img);
        const nameSpan = document.createElement("span");
        nameSpan.className = "formation-slot-name";
        nameSpan.textContent = getBodyDisplayName(idx);
        slot.appendChild(nameSpan);
      } else {
        slot.classList.add("formation-slot-empty");
        slot.innerHTML = '<span class="formation-slot-plus">+</span>';
      }
      slots.appendChild(slot);
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
    formedUnitsList.push({
      id,
      name,
      indices: [-1, -1, -1],
      energy: 0,
      avgSpeed: 0,
    });
    setFormedUnitsList([...formedUnitsList]);
    renderFormationContent();
  });

  const listEl = formationEl.querySelector("[data-formation-unit-list]")!;
  listEl.addEventListener("click", (e) => {
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (slotBtn) {
      const unitId = slotBtn.dataset.unitId;
      const slotIndex = parseInt(slotBtn.dataset.slotIndex ?? "-1", 10);
      if (unitId && slotIndex >= 0 && slotIndex <= 2) {
        openCharacterPicker(unitId, slotIndex as 0 | 1 | 2);
      }
      return;
    }
  });

  characterPickerEl.querySelector("[data-char-picker-close]")?.addEventListener("click", () => {
    closeCharacterPicker();
  });

  characterPickerEl.querySelector("[data-char-picker-grid]")?.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    const charIndex = parseInt(card.dataset.charIndex ?? "-1", 10);
    if (charIndex < 0 || !editingUnitId || editingSlotIndex === null) return;

    const unit = formedUnitsList.find((u) => u.id === editingUnitId);
    if (!unit) return;

    const newIndices: [number, number, number] = [...unit.indices];
    newIndices[editingSlotIndex] = charIndex;
    const { energy, avgSpeed } = recalcUnitStats(newIndices, bodyEnergies, bodySpeeds);

    const idx = formedUnitsList.findIndex((u) => u.id === editingUnitId);
    const updated = [...formedUnitsList];
    updated[idx] = { ...unit, indices: newIndices, energy, avgSpeed };
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
