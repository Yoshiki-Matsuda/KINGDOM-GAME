import type { InventoryItem } from "./shared/game-state";
import { getItem, getItemCount } from "./game/items";

interface PanelHeaderOptions {
  title: string;
  subtitle: string;
}

interface FacilityCardOptions {
  icon: string;
  name: string;
  description: string;
  currentLevel: number;
  isBuilding: boolean;
  isMaxLevel: boolean;
  buildCompleteAt?: number | null;
  nextEffectHtml?: string;
  costHtml?: string;
  buttonHtml?: string;
  extraClasses?: string[];
}

export function renderPanelHeader(options: PanelHeaderOptions): string {
  return `
    <div class="facility-panel-header">
      <h2>${options.title}</h2>
      <div class="selected-tile-info">${options.subtitle}
        <button type="button" class="deselect-tile-btn">✕</button>
      </div>
    </div>
  `;
}

export function renderFacilityCosts(
  cost: { itemId: string; count: number }[],
  inventory: InventoryItem[],
): string {
  return cost
    .map((entry) => {
      const item = getItem(entry.itemId);
      const have = getItemCount(inventory, entry.itemId);
      const enough = have >= entry.count;
      return `<span class="cost-item ${enough ? "" : "not-enough"}">${item?.icon ?? "?"}${have}/${entry.count}</span>`;
    })
    .join("");
}

export function renderFacilityCard(options: FacilityCardOptions): string {
  const classes = [
    "facility-card",
    options.isBuilding ? "is-building" : "",
    options.isMaxLevel ? "is-max" : "",
    ...(options.extraClasses ?? []),
  ]
    .filter(Boolean)
    .join(" ");

  return `
    <div class="${classes}">
      <div class="facility-icon">${options.icon}</div>
      <div class="facility-info">
        <div class="facility-name">${options.name}</div>
        <div class="facility-level">Lv.${options.currentLevel}${options.isMaxLevel ? " (MAX)" : ""}</div>
        <div class="facility-desc">${options.description}</div>
        ${options.isBuilding && options.buildCompleteAt ? `
          <div class="facility-building">
            建設中 <span class="facility-building-time" data-complete-at="${options.buildCompleteAt}"></span>
          </div>
        ` : ""}
        ${options.nextEffectHtml ? `
          <div class="facility-next">
            <div class="next-effect">${options.nextEffectHtml}</div>
            ${options.costHtml ? `<div class="next-cost">${options.costHtml}</div>` : ""}
            ${options.buttonHtml ?? ""}
          </div>
        ` : ""}
      </div>
    </div>
  `;
}
