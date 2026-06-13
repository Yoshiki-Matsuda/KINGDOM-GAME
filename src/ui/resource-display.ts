/**
 * 資源・通貨の表示（アイコン + 数値のみで統一）
 */

import { gameState, getLocalPlayerId } from "../store";
import { getPlayerResources, type BasicResourceType, type Resources } from "../shared/game-state";

export const RESOURCE_ICONS = {
  food: "🌾",
  wood: "🪵",
  stone: "🪨",
  iron: "⛏",
  gold: "💰",
} as const;

export type ResourceDisplayType = keyof typeof RESOURCE_ICONS;

const BASIC_RESOURCE_NAMES: Record<BasicResourceType, ResourceDisplayType> = {
  food: "food",
  wood: "wood",
  stone: "stone",
  iron: "iron",
};

export function formatResourceAmount(amount: number): string {
  return amount.toLocaleString();
}

/** アイコン + 数値のHTML */
export function renderResourceValueHtml(
  type: ResourceDisplayType,
  amount: number,
  className = "resource-value",
): string {
  return `<span class="${className}" data-resource-type="${type}"><span class="resource-icon" aria-hidden="true">${RESOURCE_ICONS[type]}</span><span class="resource-amount">${formatResourceAmount(amount)}</span></span>`;
}

/** 基本資源タイプからアイコン + 数値 */
export function renderBasicResourceHtml(
  type: BasicResourceType,
  amount: number,
  className = "resource-value",
): string {
  return renderResourceValueHtml(BASIC_RESOURCE_NAMES[type], amount, className);
}

/** ログ用: アイコン + 緑の増減表示（例: +64） */
export function renderResourceDeltaHtml(
  type: ResourceDisplayType,
  delta: number,
  className = "resource-value resource-delta",
): string {
  const sign = delta >= 0 ? "+" : "-";
  const amount = Math.abs(delta);
  return `<span class="${className}"><span class="resource-icon" aria-hidden="true">${RESOURCE_ICONS[type]}</span><span class="resource-delta-amount">${sign}${formatResourceAmount(amount)}</span></span>`;
}

/** HUD・ヘッダー用の資源バー */
export function renderResourcesHtml(className = "hud-resources"): string {
  const res = getPlayerResources(gameState, getLocalPlayerId());
  return renderResourcesBarHtml(res, className);
}

export function renderResourcesBarHtml(res: Resources, className = "hud-resources"): string {
  const items = (["food", "wood", "stone", "iron", "gold"] as const)
    .map((type) => renderResourceValueHtml(type, res[type], "resource-value resource-value--inline"))
    .join("");
  return `<span class="${className}">${items}</span>`;
}

/** ゴールドのみ（フリマヘッダー等） */
export function renderGoldHtml(amount: number, className = "resource-value"): string {
  return renderResourceValueHtml("gold", amount, className);
}
