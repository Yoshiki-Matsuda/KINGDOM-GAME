/**
 * 編成画面 — カード詳細オーバーレイ＆生産パネル
 * ロングプレス/クリックで開く魔獣詳細表示と魔獣生産UIの制御
 */

import {
  FOOD_PER_MONSTER_PRODUCE,
  getPlayerOwnedCards,
  produceMonstersAction,
} from "../shared/game-state";
import { getBodyDisplayName } from "../game/characters";
import { renderBasicResourceHtml } from "./resource-display";
import { buildFormationCardDetailHtml } from "../formation-card-detail";
import { gameState, getLocalPlayerId, ws } from "../store";
import {
  openStatAllocDialog,
  closeStatAllocDialog,
} from "./formation-stat-dialog";

// --- モジュール状態 ---
let cardDetailOpenedAt = 0;
export let openCardDetailBodySlot: number | null = null;
let pendingProduceBodySlot: number | null = null;

const CARD_DETAIL_CLOSE_DELAY_MS = 800;

export function getOpenCardDetailBodySlot(): number | null {
  return openCardDetailBodySlot;
}

export function getPendingProduceBodySlot(): number | null {
  return pendingProduceBodySlot;
}

export function clearPendingProduceBodySlot(): void {
  pendingProduceBodySlot = null;
}

// --- 生産フィードバック ---
export function findLatestProduceFeedback(bodySlot: number): string | null {
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

// --- DOM操作ヘルパー ---
export function setProduceError(contentEl: HTMLElement, message: string | null): void {
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
export function clampProduceAmountInput(input: HTMLInputElement, allowEmpty = false): number {
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

// --- パネル配線 ---
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

function wireTrainButton(
  formationEl: HTMLDivElement,
  contentEl: HTMLElement,
  bodySlot: number,
): void {
  contentEl.querySelector<HTMLButtonElement>("[data-train-open]")?.addEventListener("click", () => {
    openStatAllocDialog(formationEl, bodySlot, (slot) => refreshCardDetailContent(formationEl, slot));
  });
}

// --- カード詳細オーバーレイ制御 ---
export function refreshCardDetailContent(formationEl: HTMLDivElement, bodySlot: number): void {
  const cardDetailEl = formationEl.querySelector(".formation-card-detail-overlay");
  const contentEl = cardDetailEl?.querySelector("[data-card-detail-content]") as HTMLElement | null;
  if (!contentEl) return;
  contentEl.innerHTML = buildFormationCardDetailHtml(bodySlot);
  wireProducePanel(contentEl, bodySlot);
  wireTrainButton(formationEl, contentEl, bodySlot);
}

export function showCardDetail(formationEl: HTMLDivElement, bodySlot: number): void {
  openCardDetailBodySlot = bodySlot;
  const cardDetailEl = formationEl.querySelector(".formation-card-detail-overlay")!;
  refreshCardDetailContent(formationEl, bodySlot);
  cardDetailEl.classList.add("is-open");
  cardDetailOpenedAt = Date.now();
}

export function closeCardDetail(): void {
  openCardDetailBodySlot = null;
  closeStatAllocDialog();
  const overlay = document.querySelector(".formation-card-detail-overlay");
  overlay?.classList.remove("is-open");
}

/** formationEl に対して card-detail-overlay のクリックリスナーを登録 */
export function setupCardDetailOverlayListener(formationEl: HTMLDivElement): void {
  const cardDetailOverlay = formationEl.querySelector(".formation-card-detail-overlay");
  cardDetailOverlay?.querySelector("[data-card-detail-close]")?.addEventListener("click", closeCardDetail);
  cardDetailOverlay?.addEventListener("click", (e) => {
    if (e.target !== cardDetailOverlay) return;
    if (Date.now() - cardDetailOpenedAt < CARD_DETAIL_CLOSE_DELAY_MS) return;
    closeCardDetail();
  });
}
