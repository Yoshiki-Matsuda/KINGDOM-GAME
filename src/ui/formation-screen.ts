/**
 * 編成画面UI
 * ユニット一覧 → 行クリックで3枠表示 → 枠クリックでキャラ選択（イラスト羅列）
 */

import {
  bodyMonsterCounts, bodySpeeds,
  formedUnitsList,
  getNextFormedUnitId,
  gameState, render, ws, getLocalPlayerId,
} from "../store";
import {
  FOOD_PER_MONSTER_PRODUCE,
  allocateCardStatsAction,
  getMarchLockedCardSlots,
  getPlayerOwnedCards,
  produceMonstersAction,
} from "../shared/game-state";
import { formatStatAllocationHtml } from "../game/effective-stats";
import { renderBasicResourceHtml } from "./resource-display";
import { getBodyDisplayName, getCardRarityClass, getCharacterIllustrationPath, getCharacterStats, getUniqueIllustratedSpeciesSlots } from "../game/characters";
import { ensureDevUnit, getHomeTroops, validateFormedUnits, recalcUnitStats } from "../game/formation";
import { getEffectiveUnitCostCap, getUnitCapacity } from "../game/facility-selectors";
import { buildFormationCardDetailHtml, buildStatAllocDialogHtml } from "../formation-card-detail";
import { appendFormedUnit } from "../store-actions";
import { commitFormedUnits } from "../game/formed-units-persist";
import { escapeHtml } from "../utils";
import { renderScreenHeaderTitle } from "./screen-header";

let formationEl: HTMLDivElement;
let characterPickerEl: HTMLDivElement;
let hubEl: HTMLDivElement;
let formationModalEl: HTMLDivElement;

type FormationView = "hub" | "units" | "monsters";
type CharPickerMode = "assign" | "browse";

let formationView: FormationView = "hub";
let charPickerMode: CharPickerMode = "assign";

/** 編集中: どのユニットのどのスロットか */
let editingUnitId: string | null = null;
let editingSlotIndex: 0 | 1 | 2 | null = null;

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

function clearFormationSlot(unitId: string, slotIndex: 0 | 1 | 2): void {
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

export function createFormationElement(): HTMLDivElement {
  formationEl = document.createElement("div");
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

  hubEl = formationEl.querySelector("[data-formation-hub]") as HTMLDivElement;
  formationModalEl = formationEl.querySelector("[data-formation-modal]") as HTMLDivElement;

  characterPickerEl = document.createElement("div");
  characterPickerEl.className = "formation-char-picker-overlay";
  characterPickerEl.innerHTML = `
    <div class="formation-char-picker-modal">
      <div class="formation-char-picker-title" data-char-picker-title>魔獣一覧</div>
      <p class="formation-char-picker-desc" data-char-picker-desc hidden>タップで詳細を表示</p>
      <p class="formation-error" data-char-picker-error hidden></p>
      <div class="formation-char-picker-grid" data-char-picker-grid></div>
      <button type="button" class="formation-char-picker-close" data-char-picker-close>戻る</button>
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

  const statAllocEl = document.createElement("div");
  statAllocEl.className = "formation-stat-alloc-overlay";
  statAllocEl.innerHTML = `
    <div class="formation-stat-alloc-modal" data-stat-alloc-content></div>
  `;
  formationEl.appendChild(statAllocEl);

  setupFormationScreen();
  return formationEl;
}

function setMapPointerBlocked(blocked: boolean): void {
  const mapContainer = document.querySelector<HTMLElement>(".map-container");
  if (mapContainer) mapContainer.style.pointerEvents = blocked ? "none" : "";
}

function showFormationHubPanel(): void {
  formationView = "hub";
  hubEl.classList.add("is-active");
  formationModalEl.classList.remove("is-active");
  characterPickerEl.classList.remove("is-open");
  closeCardDetail();
}

function showFormationModalPanel(): void {
  formationView = "units";
  hubEl.classList.remove("is-active");
  formationModalEl.classList.add("is-active");
}

function openFormationOverlay(): void {
  validateFormedUnits();
  ensureDevUnit();
  editingUnitId = null;
  editingSlotIndex = null;
  setFormationError(null);
  setCharPickerError(null);
  formationEl.classList.add("is-open");
  setMapPointerBlocked(true);
}

/** ボトムメニュー「編成」→ 行き先選択 */
export function showFormationHub(): void {
  openFormationOverlay();
  charPickerMode = "assign";
  showFormationHubPanel();
  (formationEl.querySelector("[data-formation-hub='monsters']") as HTMLElement)?.focus();
}

/** ユニット編成画面を直接開く（ユニット選択などから） */
export function showFormationScreen(): void {
  openFormationOverlay();
  charPickerMode = "assign";
  showFormationModalPanel();
  characterPickerEl.classList.remove("is-open");
  renderFormationContent();
  (formationEl.querySelector("[data-formation-close]") as HTMLElement)?.focus();
}

export function closeFormationScreen(): void {
  formationEl.classList.remove("is-open");
  hubEl.classList.remove("is-active");
  formationModalEl.classList.remove("is-active");
  characterPickerEl.classList.remove("is-open");
  formationView = "hub";
  charPickerMode = "assign";
  closeCardDetail();
  setMapPointerBlocked(false);
}


/** サーバー state 更新後、編成画面を開いたままなら表示を同期 */
export function refreshFormationScreenIfOpen(): void {
  if (!formationEl?.classList.contains("is-open")) return;

  // 育成ダイアログ入力中は tick で DOM を組み直さない（数値入力がリセットされる）
  if (statAllocBodySlot !== null) return;

  validateFormedUnits();
  if (formationView === "units") {
    renderFormationContent();
  } else if (formationView === "monsters") {
    renderMonsterBrowseGrid();
  }
  if (characterPickerEl.classList.contains("is-open") && charPickerMode === "assign") {
    renderCharacterPicker();
  }
  if (openCardDetailBodySlot !== null) {
    refreshCardDetailContent(openCardDetailBodySlot);
    if (pendingProduceBodySlot === openCardDetailBodySlot) {
      const overlay = formationEl.querySelector(".formation-card-detail-content");
      const feedback = findLatestProduceFeedback(openCardDetailBodySlot);
      if (overlay && feedback) {
        setProduceError(overlay as HTMLElement, feedback.includes("生産した") ? null : feedback);
      }
      pendingProduceBodySlot = null;
    }
  }
}

function openMonsterBrowse(): void {
  formationView = "monsters";
  charPickerMode = "browse";
  editingUnitId = null;
  editingSlotIndex = null;
  setCharPickerError(null);
  hubEl.classList.remove("is-active");
  formationModalEl.classList.remove("is-active");
  updateCharPickerChrome();
  renderMonsterBrowseGrid();
  characterPickerEl.classList.add("is-open");
}

function updateCharPickerChrome(): void {
  const titleEl = characterPickerEl.querySelector("[data-char-picker-title]")!;
  const descEl = characterPickerEl.querySelector<HTMLElement>("[data-char-picker-desc]")!;
  const closeBtn = characterPickerEl.querySelector<HTMLButtonElement>("[data-char-picker-close]")!;
  if (charPickerMode === "browse") {
    titleEl.textContent = "魔獣一覧";
    descEl.hidden = false;
    closeBtn.textContent = "戻る";
  } else {
    titleEl.textContent = "魔獣を選択";
    descEl.hidden = true;
    closeBtn.textContent = "閉じる";
  }
}

function openCharacterPicker(unitId: string, slotIndex: 0 | 1 | 2): void {
  editingUnitId = unitId;
  editingSlotIndex = slotIndex;
  charPickerMode = "assign";
  setCharPickerError(null);
  updateCharPickerChrome();
  renderCharacterPicker();
  characterPickerEl.classList.add("is-open");
}

function closeCharacterPicker(): void {
  characterPickerEl.classList.remove("is-open");
  if (charPickerMode === "browse") {
    charPickerMode = "assign";
    showFormationHubPanel();
    return;
  }
  editingUnitId = null;
  editingSlotIndex = null;
}

let cardDetailOpenedAt = 0;
let openCardDetailBodySlot: number | null = null;
let statAllocBodySlot: number | null = null;
const CARD_DETAIL_CLOSE_DELAY_MS = 800;

function setFormationError(message: string | null): void {
  const el = formationEl.querySelector<HTMLParagraphElement>("[data-formation-error]");
  if (!el) return;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

function setCharPickerError(message: string | null): void {
  const el = characterPickerEl.querySelector<HTMLParagraphElement>("[data-char-picker-error]");
  if (!el) return;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

let pendingProduceBodySlot: number | null = null;

function findLatestProduceFeedback(bodySlot: number): string | null {
  const owned = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const cardId = owned[bodySlot];
  if (cardId == null) return null;
  const cardName = getBodyDisplayName(cardId);
  const logs = gameState.log ?? [];
  for (let i = logs.length - 1; i >= Math.max(0, logs.length - 8); i--) {
    const line = logs[i].message ?? "";
    if (line.includes("遠征中の魔獣は生産できません")) return line;
    if (line.includes("食料が足りません")) return line;
    if (line.includes("これ以上魔獣を生産できません")) return line;
    if (line.includes("無効な魔獣スロット")) return line;
    if (line.includes(`「${cardName}」に魔獣を`) && line.includes("生産した")) return line;
  }
  return null;
}

function setProduceError(contentEl: HTMLElement, message: string | null): void {
  const el = contentEl.querySelector<HTMLParagraphElement>("[data-produce-error]");
  if (!el) return;
  if (message) {
    el.textContent = message;
    el.hidden = false;
  } else {
    el.hidden = true;
  }
}

function getProduceMaxAllowed(input: HTMLInputElement): number {
  const maxAttr = parseInt(input.getAttribute("max") ?? "0", 10);
  return Number.isFinite(maxAttr) && maxAttr > 0 ? maxAttr : 0;
}

/** 生産数入力を 1〜max に収め、正規化後の値を返す */
function clampProduceAmountInput(input: HTMLInputElement, allowEmpty = false): number {
  const maxAllowed = getProduceMaxAllowed(input);
  const raw = input.value.trim();

  if (raw === "") {
    if (!allowEmpty) input.value = "1";
    return 1;
  }

  let amount = parseInt(raw, 10);
  if (!Number.isFinite(amount)) {
    input.value = "1";
    return 1;
  }
  if (amount < 1) amount = 1;
  if (maxAllowed > 0 && amount > maxAllowed) amount = maxAllowed;

  const normalized = String(amount);
  if (input.value !== normalized) input.value = normalized;
  return amount;
}

function updateProduceCostDisplay(input: HTMLInputElement, costEl: HTMLElement, allowEmpty = false): void {
  const amount = clampProduceAmountInput(input, allowEmpty);
  const foodCost = amount * FOOD_PER_MONSTER_PRODUCE;
  costEl.innerHTML = renderBasicResourceHtml(
    "food",
    foodCost,
    "resource-value formation-produce-cost-value",
  );
}

function wireProducePanel(contentEl: HTMLElement, bodySlot: number): void {
  const panel = contentEl.querySelector<HTMLElement>("[data-produce-panel]");
  const startBtn = contentEl.querySelector<HTMLButtonElement>("[data-produce-start]");
  const input = contentEl.querySelector<HTMLInputElement>("[data-produce-amount]");
  const costEl = contentEl.querySelector<HTMLElement>("[data-produce-cost]");
  if (!panel || !startBtn || !input) return;

  const openPanel = (): void => {
    panel.hidden = false;
    startBtn.hidden = true;
    input.value = "1";
    if (costEl) updateProduceCostDisplay(input, costEl);
    input.focus();
    input.select();
  };

  const closePanel = (): void => {
    panel.hidden = true;
    startBtn.hidden = false;
  };

  const sendProduce = (): void => {
    setProduceError(contentEl, null);
    if (ws?.readyState !== WebSocket.OPEN) {
      setProduceError(contentEl, "サーバーに接続されていません。接続後にもう一度お試しください。");
      return;
    }
    const n = clampProduceAmountInput(input);
    if (costEl) updateProduceCostDisplay(input, costEl);
    pendingProduceBodySlot = bodySlot;
    ws.send(JSON.stringify(produceMonstersAction(bodySlot, n)));
    closePanel();
  };

  startBtn.addEventListener("click", openPanel);
  contentEl.querySelector<HTMLButtonElement>("[data-produce-cancel]")?.addEventListener("click", closePanel);
  contentEl.querySelector<HTMLButtonElement>("[data-produce-submit]")?.addEventListener("click", sendProduce);
  if (costEl) {
    input.addEventListener("input", () => updateProduceCostDisplay(input, costEl, true));
    input.addEventListener("change", () => updateProduceCostDisplay(input, costEl));
    input.addEventListener("blur", () => updateProduceCostDisplay(input, costEl));
  }
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      sendProduce();
    }
  });
}

function wireStatAllocPanel(contentEl: HTMLElement, bodySlot: number): void {
  const spendEl = contentEl.querySelector<HTMLElement>("[data-stat-spend]");
  const unspentEl = contentEl.querySelector<HTMLElement>("[data-stat-unspent]");
  const errorEl = contentEl.querySelector<HTMLElement>("[data-stat-alloc-error]");
  const submitBtn = contentEl.querySelector<HTMLButtonElement>("[data-stat-alloc-submit]");
  const cancelBtn = contentEl.querySelector<HTMLButtonElement>("[data-stat-alloc-cancel]");
  const inputs = contentEl.querySelectorAll<HTMLInputElement>("[data-stat-key]");
  if (!spendEl || !unspentEl || !submitBtn) return;

  const maxSpend = parseInt(unspentEl.textContent ?? "0", 10) || 0;

  const readSpend = (): number => {
    let total = 0;
    for (const input of inputs) {
      total += Math.max(0, parseInt(input.value, 10) || 0);
    }
    return total;
  };

  const refreshPreviews = (): void => {
    for (const input of inputs) {
      const key = input.dataset.statKey;
      const preview = contentEl.querySelector<HTMLElement>(`[data-stat-preview="${key}"]`);
      if (!preview) continue;
      const core = parseInt(input.dataset.statCore ?? "0", 10) || 0;
      const allocated = parseInt(input.dataset.statAllocated ?? "0", 10) || 0;
      const pending = Math.max(0, parseInt(input.value, 10) || 0);
      preview.innerHTML = formatStatAllocationHtml(core, allocated, pending);
    }
  };

  const refreshSpend = (): void => {
    const total = readSpend();
    spendEl.textContent = String(total);
    refreshPreviews();
    if (errorEl) {
      if (total > maxSpend) {
        errorEl.hidden = false;
        errorEl.textContent = `ポイントが足りません（最大 ${maxSpend}）。`;
      } else {
        errorEl.hidden = true;
        errorEl.textContent = "";
      }
    }
    submitBtn.disabled = total <= 0 || total > maxSpend;
  };

  for (const input of inputs) {
    input.addEventListener("input", refreshSpend);
    input.addEventListener("change", refreshSpend);
  }

  submitBtn.addEventListener("click", () => {
    const delta = {
      speed: 0,
      attack: 0,
      intelligence: 0,
      defense: 0,
      magic_defense: 0,
    };
    for (const input of inputs) {
      const key = input.dataset.statKey as keyof typeof delta | undefined;
      if (!key) continue;
      delta[key] = Math.max(0, parseInt(input.value, 10) || 0);
    }
    const total = Object.values(delta).reduce((a, b) => a + b, 0);
    if (total <= 0 || total > maxSpend) {
      refreshSpend();
      return;
    }
    if (!ws) return;
    ws.send(JSON.stringify(allocateCardStatsAction(bodySlot, delta)));
    closeStatAllocDialog();
    if (openCardDetailBodySlot === bodySlot) {
      refreshCardDetailContent(bodySlot);
    }
  });

  cancelBtn?.addEventListener("click", closeStatAllocDialog);

  refreshSpend();
}

function refreshCardDetailContent(bodySlot: number): void {
  const cardDetailEl = formationEl.querySelector(".formation-card-detail-overlay");
  const contentEl = cardDetailEl?.querySelector("[data-card-detail-content]") as HTMLElement | null;
  if (!contentEl) return;
  contentEl.innerHTML = buildFormationCardDetailHtml(bodySlot);
  wireProducePanel(contentEl, bodySlot);
  wireTrainButton(contentEl, bodySlot);
}

function refreshStatAllocDialog(bodySlot: number): void {
  const overlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  const contentEl = overlay?.querySelector("[data-stat-alloc-content]") as HTMLElement | null;
  if (!contentEl) return;
  contentEl.innerHTML = buildStatAllocDialogHtml(bodySlot);
  wireStatAllocPanel(contentEl, bodySlot);
}

function openStatAllocDialog(bodySlot: number): void {
  statAllocBodySlot = bodySlot;
  const overlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  if (!overlay) return;
  refreshStatAllocDialog(bodySlot);
  overlay.classList.add("is-open");
}

function closeStatAllocDialog(): void {
  statAllocBodySlot = null;
  formationEl.querySelector(".formation-stat-alloc-overlay")?.classList.remove("is-open");
}

function wireTrainButton(contentEl: HTMLElement, bodySlot: number): void {
  contentEl.querySelector<HTMLButtonElement>("[data-train-open]")?.addEventListener("click", () => {
    openStatAllocDialog(bodySlot);
  });
}

function showCardDetail(bodySlot: number): void {
  openCardDetailBodySlot = bodySlot;
  const cardDetailEl = formationEl.querySelector(".formation-card-detail-overlay")!;
  refreshCardDetailContent(bodySlot);
  cardDetailEl.classList.add("is-open");
  cardDetailOpenedAt = Date.now();
}

function closeCardDetail(): void {
  openCardDetailBodySlot = null;
  closeStatAllocDialog();
  formationEl.querySelector(".formation-card-detail-overlay")?.classList.remove("is-open");
}

function renderMonsterBrowseGrid(): void {
  const gridEl = characterPickerEl.querySelector("[data-char-picker-grid]")!;
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
  const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
  const marchLockedSlots = getMarchLockedCardSlots(gameState, getLocalPlayerId());
  // 編集中ユニットの他枠で使用中の魔獣種（同種は1ユニットに1体まで）
  const cardIdsInUnit = new Set(
    unit.indices
      .filter((idx, slot) => idx >= 0 && slot !== editingSlotIndex)
      .map((idx) => ownedCards[idx] ?? idx),
  );

  for (let i = 0; i < homeTroops; i++) {
    const used = usedByOthers.has(i);
    const isCurrentSlot = unit.indices[editingSlotIndex] === i;
    const sameSpeciesInUnit = cardIdsInUnit.has(ownedCards[i] ?? i);
    const onMarch = marchLockedSlots.has(i);
    const canSelect =
      (!used || isCurrentSlot) && (!sameSpeciesInUnit || isCurrentSlot) && (!onMarch || isCurrentSlot);

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

function setupFormationScreen(): void {
  formationEl.querySelector("[data-formation-hub='monsters']")?.addEventListener("click", () => {
    openMonsterBrowse();
  });

  formationEl.querySelector("[data-formation-hub='units']")?.addEventListener("click", () => {
    showFormationModalPanel();
    renderFormationContent();
    (formationEl.querySelector("[data-formation-close]") as HTMLElement)?.focus();
  });

  formationEl.querySelector("[data-formation-hub-close]")?.addEventListener("click", () => {
    closeFormationScreen();
    render();
  });

  formationEl.querySelector("[data-formation-close]")?.addEventListener("click", () => {
    if (formationView === "units") {
      showFormationHubPanel();
      return;
    }
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
    didLongPress = false;
    if ((e.target as HTMLElement).closest(".formation-slot-unassign")) return;
    const slotBtn = (e.target as HTMLElement).closest<HTMLButtonElement>("button.formation-slot");
    if (!slotBtn || slotBtn.classList.contains("formation-slot-empty")) return;
    const charIndex = slotBtn.dataset.charIndex;
    if (!charIndex) return;
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
      didLongPress = false;
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
      if (didLongPress) {
        didLongPress = false;
        return;
      }
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

  const statAllocOverlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  statAllocOverlay?.addEventListener("click", (e) => {
    if (e.target === statAllocOverlay) closeStatAllocDialog();
  });

  const gridEl = characterPickerEl.querySelector("[data-char-picker-grid]")!;
  let pickerLongPressTimer: number | null = null;
  let pickerDidLongPress = false;

  gridEl.addEventListener("pointerdown", (e) => {
    pickerDidLongPress = false;
    const card = (e.target as HTMLElement).closest<HTMLButtonElement>("button[data-char-index]");
    if (!card || card.disabled) return;
    const charIndex = card.dataset.charIndex;
    if (!charIndex) return;
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
      pickerDidLongPress = false;
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
    if (pickerDidLongPress) {
      pickerDidLongPress = false;
      return;
    }
    const charIndex = parseInt(card.dataset.charIndex ?? "-1", 10);
    if (charIndex < 0) return;

    if (charPickerMode === "browse") {
      showCardDetail(charIndex);
      return;
    }

    if (!editingUnitId || editingSlotIndex === null) return;

    const unit = formedUnitsList.find((u) => u.id === editingUnitId);
    if (!unit) return;

    const ownedCards = getPlayerOwnedCards(gameState, getLocalPlayerId());
    const pickedCardId = ownedCards[charIndex] ?? charIndex;
    const duplicateSpecies = unit.indices.some(
      (idx, slot) => slot !== editingSlotIndex && idx >= 0 && (ownedCards[idx] ?? idx) === pickedCardId,
    );
    if (duplicateSpecies) {
      setCharPickerError("同じ種類の魔獣は1ユニットに1体までです。別の魔獣を選んでください。");
      return;
    }

    const newIndices: [number, number, number] = [...unit.indices];
    newIndices[editingSlotIndex] = charIndex;
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

    const idx = formedUnitsList.findIndex((u) => u.id === editingUnitId);
    const updated = [...formedUnitsList];
    updated[idx] = { ...unit, indices: newIndices, monster_count, avgSpeed };
    commitFormedUnits(updated);

    closeCharacterPicker();
    renderFormationContent();
  });

  characterPickerEl.addEventListener("click", (e) => {
    if (e.target === characterPickerEl) closeCharacterPicker();
  });

  formationEl.addEventListener("click", (e) => {
    if (e.target !== formationEl) return;
    if (formationView === "hub") {
      closeFormationScreen();
      render();
    } else if (formationView === "units") {
      showFormationHubPanel();
    }
  });
}
