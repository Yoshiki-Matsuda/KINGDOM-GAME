/**
 * 編成画面 — ハブ/ユニットビュー
 * ユニット一覧描画、ユニット追加・スロット操作、ハブ画面切り替え
 */

import {
  bodyMonsterCounts, bodySpeeds,
  formedUnitsList, getLocalPlayerId,
  gameState,
} from "../store";
import {
  getPlayerOwnedCards,
} from "../shared/game-state";
import { getBodyDisplayName, getCardRarityClass, getCharacterIllustrationPath, getCharacterStats } from "../game/characters";
import { ensureDevUnit, validateFormedUnits, recalcUnitStats, getHomeTroops } from "../game/formation";
import { getEffectiveUnitCostCap, getUnitCapacity } from "../game/facility-selectors";
import { commitFormedUnits } from "../game/formed-units-persist";
import { escapeHtml } from "../utils";
import { renderScreenHeaderTitle } from "./screen-header";
import { closeCardDetail } from "./formation-card-view";
import { shared, setMapPointerBlocked } from "./formation-shared";

export function createFormationElement(): HTMLDivElement {
  const formationEl = document.createElement("div");
  formationEl.className = "formation-overlay";
  formationEl.innerHTML = `
    <div class="formation-hub" data-formation-hub>
      <div class="formation-hub-modal">
        <div class="formation-hub-title">${renderScreenHeaderTitle("formation", "編成")}</div>
        <p class="formation-hub-desc">行き先を選んでください</p>
        <div class="formation-hub-actions">
          <button type="button" class="formation-hub-btn" data-formation-hub="monsters">
            <span class="formation-hub-btn-label">魔獣一覧</span>
            <span class="formation-hub-btn-hint">所持魔獣の確認・生産</span>
          </button>
          <button type="button" class="formation-hub-btn" data-formation-hub="units">
            <span class="formation-hub-btn-label">ユニット編成</span>
            <span class="formation-hub-btn-hint">戦闘ユニットの編成</span>
          </button>
        </div>
        <button type="button" class="formation-hub-close" data-formation-hub-close>閉じる</button>
      </div>
    </div>
    <div class="formation-modal" data-formation-modal>
      <div class="formation-title">${renderScreenHeaderTitle("formation", "ユニット編成")}</div>
      <p class="formation-error" data-formation-error hidden></p>
      <div class="formation-desc">枠をクリックしてキャラを選択。「外す」で枠を空に戻せます</div>
      <div class="formation-troops" data-formation-troops>本拠地: 0 体</div>
      <div class="formation-unit-list" data-formation-unit-list></div>
      <button type="button" class="formation-add-unit" data-formation-add-unit>新規ユニットを追加</button>
      <button type="button" class="formation-close" data-formation-close>戻る</button>
    </div>
  `;

  shared.formationEl = formationEl;
  shared.hubEl = formationEl.querySelector("[data-formation-hub]") as HTMLDivElement;
  shared.formationModalEl = formationEl.querySelector("[data-formation-modal]") as HTMLDivElement;

  shared.characterPickerEl = document.createElement("div");
  shared.characterPickerEl.className = "formation-char-picker-overlay";
  shared.characterPickerEl.innerHTML = `
    <div class="formation-char-picker-modal">
      <div class="formation-char-picker-title" data-char-picker-title>魔獣一覧</div>
      <p class="formation-char-picker-desc" data-char-picker-desc hidden>タップで詳細を表示</p>
      <p class="formation-error" data-char-picker-error hidden></p>
      <div class="formation-char-picker-grid" data-char-picker-grid></div>
      <button type="button" class="formation-char-picker-close" data-char-picker-close>戻る</button>
    </div>
  `;
  formationEl.appendChild(shared.characterPickerEl);

  shared.cardDetailEl = document.createElement("div");
  shared.cardDetailEl.className = "formation-card-detail-overlay";
  shared.cardDetailEl.innerHTML = `
    <div class="formation-card-detail-modal">
      <div class="formation-card-detail-content" data-card-detail-content></div>
      <button type="button" class="formation-card-detail-close" data-card-detail-close>閉じる</button>
    </div>
  `;
  formationEl.appendChild(shared.cardDetailEl);

  shared.statAllocEl = document.createElement("div");
  shared.statAllocEl.className = "formation-stat-alloc-overlay";
  shared.statAllocEl.innerHTML = `
    <div class="formation-stat-alloc-modal" data-stat-alloc-content></div>
  `;
  formationEl.appendChild(shared.statAllocEl);

  return formationEl;
}

/** ユニット上限（施設ボーナス込み） */
function getMaxUnits(): number {
  return Math.max(1, 1 + getUnitCapacity(gameState));
}

function unitFilledCostSum(indices: [number, number, number]): number {
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  let s = 0;
  for (const i of indices) {
    if (i >= 0) s += getCharacterStats(owned[i] ?? 0).cost;
  }
  return s;
}

export function clearFormationSlot(unitId: string, slotIndex: 0 | 1 | 2): void {
  const unit = formedUnitsList.find((u) => u.id === unitId);
  if (!unit || unit.indices[slotIndex] < 0) return;
  const newIndices: [number, number, number] = [...unit.indices];
  newIndices[slotIndex] = -1;
  const { monster_count, avgSpeed } = recalcUnitStats(newIndices, bodyMonsterCounts, bodySpeeds);
  const idx = formedUnitsList.findIndex((u) => u.id === unitId);
  const updated = [...formedUnitsList];
  updated[idx] = { ...unit, indices: newIndices, monster_count, avgSpeed };
  commitFormedUnits(updated);
  renderFormationContent();
}

export function showFormationHubPanel(): void {
  shared.formationView = "hub";
  shared.hubEl?.classList.add("is-active");
  shared.formationModalEl?.classList.remove("is-active");
  shared.characterPickerEl?.classList.remove("is-open");
  closeCardDetail();
}

export function showFormationModalPanel(): void {
  shared.formationView = "units";
  shared.hubEl?.classList.remove("is-active");
  shared.formationModalEl?.classList.add("is-active");
}

export function openFormationOverlay(): void {
  validateFormedUnits();
  ensureDevUnit();
  shared.editingUnitId = null;
  shared.editingSlotIndex = null;
  setFormationError(null);
  shared.formationEl?.classList.add("is-open");
  setMapPointerBlocked(true);
}

/** ボトムメニュー「編成」→ 行き先選択 */
export function showFormationHub(): void {
  openFormationOverlay();
  shared.charPickerMode = "assign";
  showFormationHubPanel();
  (shared.formationEl?.querySelector("[data-formation-hub='monsters']") as HTMLElement)?.focus();
}

/** ユニット編成画面を直接開く（ユニット選択などから） */
export function showFormationScreen(): void {
  openFormationOverlay();
  shared.charPickerMode = "assign";
  showFormationModalPanel();
  shared.characterPickerEl?.classList.remove("is-open");
  renderFormationContent();
  (shared.formationEl?.querySelector("[data-formation-close]") as HTMLElement)?.focus();
}

export function closeFormationScreen(): void {
  shared.formationEl?.classList.remove("is-open");
  shared.hubEl?.classList.remove("is-active");
  shared.formationModalEl?.classList.remove("is-active");
  shared.characterPickerEl?.classList.remove("is-open");
  shared.formationView = "hub";
  shared.charPickerMode = "assign";
  closeCardDetail();
  setMapPointerBlocked(false);
}

function setFormationError(message: string | null): void {
  const el = shared.formationEl?.querySelector<HTMLParagraphElement>("[data-formation-error]");
  if (!el) return;
  if (message) { el.textContent = message; el.hidden = false; }
  else { el.hidden = true; }
}

export function renderFormationContent(): void {
  const homeTroops = getHomeTroops();
  const maxUnits = getMaxUnits();
  const costCap = getEffectiveUnitCostCap(gameState);
  const troopsEl = shared.formationEl?.querySelector("[data-formation-troops]")!;
  const listEl = shared.formationEl?.querySelector("[data-formation-unit-list]")!;
  const addBtn = shared.formationEl?.querySelector<HTMLButtonElement>("[data-formation-add-unit]")!;

  troopsEl.textContent = `本拠地: ${homeTroops} 体・コスト上限 ${costCap.toFixed(1)}`;
  addBtn.disabled = formedUnitsList.length >= maxUnits;

  listEl.innerHTML = "";
  const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
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
    const POSITION_RANGE_HINT = ["射程1+", "射程2+", "射程3で瞄われる"] as const;
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
        nameSpan.className = `formation-slot-name ${getCardRarityClass(cardId)}`;
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
