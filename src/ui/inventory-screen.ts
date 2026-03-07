/**
 * インベントリ画面
 */

import { gameState } from "../store";
import { getItem, getRarityColor } from "../game/items";

let inventoryEl: HTMLDivElement;
let itemListEl: HTMLDivElement;
let selectedCategory: "all" | "material" | "skill_book" | "special" = "all";

export function createInventoryElement(): HTMLDivElement {
  inventoryEl = document.createElement("div");
  inventoryEl.className = "inventory-screen";
  inventoryEl.style.display = "none";

  inventoryEl.innerHTML = `
    <div class="inventory-header">
      <h2>インベントリ</h2>
    </div>
    <div class="inventory-categories">
      <button type="button" class="inventory-category-btn is-active" data-category="all">全て</button>
      <button type="button" class="inventory-category-btn" data-category="material">素材</button>
      <button type="button" class="inventory-category-btn" data-category="skill_book">スキル書</button>
      <button type="button" class="inventory-category-btn" data-category="special">特殊</button>
    </div>
    <div class="inventory-list"></div>
  `;

  itemListEl = inventoryEl.querySelector(".inventory-list") as HTMLDivElement;

  inventoryEl.querySelectorAll(".inventory-category-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      selectedCategory = (btn as HTMLElement).dataset.category as typeof selectedCategory;
      inventoryEl.querySelectorAll(".inventory-category-btn").forEach(b => b.classList.remove("is-active"));
      btn.classList.add("is-active");
      renderInventory();
    });
  });

  return inventoryEl;
}

export function renderInventory(): void {
  const inventory = gameState.inventory ?? [];

  // カテゴリでフィルタ
  const filtered = inventory.filter(item => {
    if (selectedCategory === "all") return true;
    const def = getItem(item.item_id);
    return def?.category === selectedCategory;
  });

  // レアリティ順（legendary > epic > rare > uncommon > common）でソート
  const rarityOrder = { legendary: 0, epic: 1, rare: 2, uncommon: 3, common: 4 };
  filtered.sort((a, b) => {
    const defA = getItem(a.item_id);
    const defB = getItem(b.item_id);
    const orderA = rarityOrder[defA?.rarity ?? "common"];
    const orderB = rarityOrder[defB?.rarity ?? "common"];
    return orderA - orderB;
  });

  if (filtered.length === 0) {
    itemListEl.innerHTML = `<div class="inventory-empty">アイテムがありません</div>`;
    return;
  }

  itemListEl.innerHTML = filtered.map(item => {
    const def = getItem(item.item_id);
    if (!def) return "";
    
    const rarityColor = getRarityColor(def.rarity);
    return `
      <div class="inventory-item" style="border-left-color: ${rarityColor}">
        <div class="item-icon">${def.icon}</div>
        <div class="item-info">
          <div class="item-name" style="color: ${rarityColor}">${def.name}</div>
          <div class="item-desc">${def.description}</div>
        </div>
        <div class="item-count">×${item.count}</div>
      </div>
    `;
  }).join("");
}

export function showInventoryScreen(): void {
  renderInventory();
}
