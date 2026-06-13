/**
 * 資源表示の増減フラッシュ（各処理に手を入れず、描画時の値変化を監視）
 */

import type { Resources } from "../shared/game-state";

const TRACKED_RESOURCE_TYPES = ["food", "wood", "stone", "iron", "gold"] as const;

const snapshots = new WeakMap<ParentNode, Resources>();

/** 前回描画値と比較し、増減した資源に一瞬だけフラッシュクラスを付与 */
export function syncResourceChangeFlashes(container: ParentNode, current: Resources): void {
  const previous = snapshots.get(container) ?? null;
  snapshots.set(container, { ...current });

  if (!previous) return;

  for (const type of TRACKED_RESOURCE_TYPES) {
    const delta = current[type] - previous[type];
    if (delta === 0) continue;

    const el = container.querySelector<HTMLElement>(`[data-resource-type="${type}"]`);
    if (!el) continue;

    el.classList.remove("resource-flash-up", "resource-flash-down");
    void el.offsetWidth;
    el.classList.add(delta > 0 ? "resource-flash-up" : "resource-flash-down");

    const onEnd = (): void => {
      el.classList.remove("resource-flash-up", "resource-flash-down");
      el.removeEventListener("animationend", onEnd);
    };
    el.addEventListener("animationend", onEnd);
  }
}
