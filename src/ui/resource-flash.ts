/**
 * 資源表示の増減フラッシュ（各処理に手を入れず、描画時の値変化を監視）
 */

import type { Resources } from "../shared/game-state";

const TRACKED_RESOURCE_TYPES = ["food", "wood", "stone", "iron", "gold"] as const;

// DOM要素が毎回再作成されるため、グローバルで前回値を保持
let previousResources: Resources | null = null;

/** 要素ごとのアニメーション世代管理 */
const animationGenerations = new WeakMap<HTMLElement, number>();

/** 要素ごとのanimationendリスナーを保存して削除用 */
const listeners = new WeakMap<HTMLElement, EventListener>();

let globalGeneration = 0;

/** 前回描画値と比較し、増減した資源に一瞬だけフラッシュクラスを付与 */
export function syncResourceChangeFlashes(container: ParentNode, current: Resources): void {
  const previous = previousResources;
  previousResources = { ...current };

  if (!previous) return;

  // 各フレームでgenerationをインクリメント
  globalGeneration++;

  for (const type of TRACKED_RESOURCE_TYPES) {
    const delta = current[type] - previous[type];
    if (delta === 0) continue;

    const el = container.querySelector<HTMLElement>(`[data-resource-type="${type}"]`);
    if (!el) continue;
    const amount = el.querySelector<HTMLElement>(".resource-amount");
    if (!amount) continue;

    animationGenerations.set(amount, globalGeneration);

    amount.classList.remove("resource-flash-up", "resource-flash-down");
    void amount.offsetWidth;
    amount.classList.add(delta > 0 ? "resource-flash-up" : "resource-flash-down");

    const thisGen = globalGeneration;
    const onEnd = (): void => {
      if (animationGenerations.get(amount) !== thisGen) return;
      amount.classList.remove("resource-flash-up", "resource-flash-down");
      amount.removeEventListener("animationend", onEnd);
      listeners.delete(amount);
    };
    // 既存のリスナーを削除してから再登録
    const prev = listeners.get(amount);
    if (prev) {
      amount.removeEventListener("animationend", prev);
    }
    listeners.set(amount, onEnd);
    amount.addEventListener("animationend", onEnd);
  }
}
