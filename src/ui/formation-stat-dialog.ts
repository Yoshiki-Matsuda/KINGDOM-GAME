/**
 * 編成画面 — ステータス配分ダイアログ
 * 育成ボタンから開くポイント振り分けUIの制御
 */

import { allocateCardStatsAction } from "../shared/game-state";
import { formatStatAllocationHtml } from "../game/effective-stats";
import { buildStatAllocDialogHtml } from "../formation-card-detail";
import { ws } from "../store";

/** 現在ダイアログを開いているbodySlot（null = 閉じている） */
export let statAllocBodySlot: number | null = null;

export function getStatAllocBodySlot(): number | null {
  return statAllocBodySlot;
}

export function openStatAllocDialog(
  formationEl: HTMLDivElement,
  bodySlot: number,
  refreshCardDetail?: (slot: number) => void,
): void {
  statAllocBodySlot = bodySlot;
  const overlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  if (!overlay) return;
  refreshStatAllocDialog(formationEl, bodySlot, refreshCardDetail);
  overlay.classList.add("is-open");
}

export function closeStatAllocDialog(): void {
  statAllocBodySlot = null;
  const overlay = document.querySelector(".formation-stat-alloc-overlay");
  overlay?.classList.remove("is-open");
}

export function refreshStatAllocDialog(
  formationEl: HTMLDivElement,
  bodySlot: number,
  refreshCardDetail?: (slot: number) => void,
): void {
  const overlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  const contentEl = overlay?.querySelector("[data-stat-alloc-content]") as HTMLElement | null;
  if (!contentEl) return;
  contentEl.innerHTML = buildStatAllocDialogHtml(bodySlot);
  wireStatAllocPanel(contentEl, bodySlot, refreshCardDetail);
}

function wireStatAllocPanel(
  contentEl: HTMLElement,
  bodySlot: number,
  refreshCardDetail?: (slot: number) => void,
): void {
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
    if (refreshCardDetail) {
      refreshCardDetail(bodySlot);
    }
  });

  cancelBtn?.addEventListener("click", closeStatAllocDialog);

  refreshSpend();
}

/** formationEl に対して stat-alloc-overlay の背景クリックを登録 */
export function setupStatAllocOverlayListener(formationEl: HTMLDivElement): void {
  const statAllocOverlay = formationEl.querySelector(".formation-stat-alloc-overlay");
  statAllocOverlay?.addEventListener("click", (e) => {
    if (e.target === statAllocOverlay) closeStatAllocDialog();
  });
}
