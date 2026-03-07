/**
 * 画面下部のメニューバー
 */

import { setCurrentScreen, currentScreen, render } from "../store";
import { showHomeScreen } from "./home-screen";
import { showInventoryScreen } from "./inventory-screen";

let menuBar: HTMLDivElement;

export function createBottomMenuElement(): HTMLDivElement {
  menuBar = document.createElement("div");
  menuBar.className = "bottom-menu";
  
  menuBar.innerHTML = `
    <button class="bottom-menu-item" data-action="home" title="本拠地">
      <span class="bottom-menu-icon">🏠</span>
      <span class="bottom-menu-label">本拠地</span>
    </button>
    <button class="bottom-menu-item" data-action="inventory" title="インベントリ">
      <span class="bottom-menu-icon">🎒</span>
      <span class="bottom-menu-label">所持品</span>
    </button>
    <button class="bottom-menu-item" data-action="history" title="戦闘履歴">
      <span class="bottom-menu-icon">📜</span>
      <span class="bottom-menu-label">履歴</span>
    </button>
    <button class="bottom-menu-item" data-action="map" title="マップ">
      <span class="bottom-menu-icon">🗺️</span>
      <span class="bottom-menu-label">マップ</span>
    </button>
  `;
  
  menuBar.addEventListener("click", (e) => {
    const btn = (e.target as HTMLElement).closest(".bottom-menu-item") as HTMLElement;
    if (!btn) return;
    
    const action = btn.dataset.action;
    
    switch (action) {
      case "home":
        showHomeScreen();
        break;
      case "inventory":
        setCurrentScreen("inventory");
        showInventoryScreen();
        render();
        break;
      case "history":
        setCurrentScreen("history");
        render();
        break;
      case "map":
        setCurrentScreen("map");
        render();
        break;
    }
    
    updateActiveState();
  });
  
  return menuBar;
}

function updateActiveState(): void {
  const items = menuBar.querySelectorAll(".bottom-menu-item");
  items.forEach((item) => {
    const action = (item as HTMLElement).dataset.action;
    item.classList.remove("is-active");
    
    if (action === "home" && currentScreen === "home") {
      item.classList.add("is-active");
    } else if (action === "map" && currentScreen === "map") {
      item.classList.add("is-active");
    } else if (action === "history" && currentScreen === "history") {
      item.classList.add("is-active");
    } else if (action === "inventory" && currentScreen === "inventory") {
      item.classList.add("is-active");
    }
  });
}

export function updateBottomMenu(): void {
  updateActiveState();
}
