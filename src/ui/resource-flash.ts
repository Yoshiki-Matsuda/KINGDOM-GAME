/**
 * 資源表示の増減フラッシュ（各処理に手を入れず、描画時の値変化を監視）
 *
 * hud.ts が資源バーの DOM を再生成しないようになったため、
 * アニメーションクラスが自然に維持される。シンプルに値変化を検出して
 * クラスを切り替えるだけにする。
 */

import type { Resources } from "../shared/game-state";

const TRACKED_RESOURCE_TYPES = ["food", "wood", "stone", "iron", "gold"] as const;

let previousResources: Resources | null = null;

export function syncResourceChangeFlashes(container: ParentNode, current: Resources): void {
  if (!previousResources) {
    previousResources = { ...current };
    return;
  }

  for (const type of TRACKED_RESOURCE_TYPES) {
    const delta = current[type] - previousResources[type];
    if (delta === 0) continue;

    previousResources[type] = current[type];
    const direction = delta > 0 ? "up" : "down";

    const el = container.querySelector<HTMLElement>(`[data-resource-type="${type}"]`);
    if (!el) continue;
    const amount = el.querySelector<HTMLElement>(".resource-amount");
    if (!amount) continue;

    amount.classList.remove("resource-flash-up", "resource-flash-down");
    void amount.offsetWidth;
    amount.classList.add(`resource-flash-${direction}`);
  }
}
