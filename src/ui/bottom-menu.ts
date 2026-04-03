/**
 * 画面下部のメニューバー
 */

import { setCurrentScreen, currentScreen, render } from "../store";
import { showHomeScreen } from "./home-screen";
import { showInventoryScreen } from "./inventory-screen";
import { showFormationScreen } from "./formation-screen";
import { showFleaMarketScreen } from "./flea-market-screen";
import { showAllianceScreen } from "./alliance-screen";
import { showStatusScreen } from "./status-screen";

let menuBar: HTMLDivElement;

export function createBottomMenuElement(): HTMLDivElement {
  menuBar = document.createElement("div");
  menuBar.className = "bottom-menu";
  
  menuBar.innerHTML = `
    <button class="bottom-menu-item" data-action="home" title="城下町">
      <span class="bottom-menu-icon"><img src="/icons/menu-home.png" alt="城下町" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="map" title="世界地図">
      <span class="bottom-menu-icon"><img src="/icons/menu-map.png" alt="地図" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="formation" title="ユニット編成">
      <span class="bottom-menu-icon"><img src="/icons/menu-formation.png" alt="編成" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="alliance" title="同盟">
      <span class="bottom-menu-icon"><img src="/icons/menu-alliance.png" alt="同盟" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="market" title="取引所">
      <span class="bottom-menu-icon"><img src="/icons/menu-market.png" alt="取引所" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="history" title="戦歴">
      <span class="bottom-menu-icon"><img src="/icons/menu-history.png" alt="戦歴" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="status" title="ステータス">
      <span class="bottom-menu-icon"><img src="/icons/menu-status.png" alt="情報" class="game-icon"></span>
    </button>
    <button class="bottom-menu-item" data-action="inventory" title="所持品">
      <span class="bottom-menu-icon"><img src="/icons/menu-inventory.png" alt="所持品" class="game-icon"></span>
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
      case "map":
        setCurrentScreen("map");
        render();
        break;
      case "formation":
        showFormationScreen();
        break;
      case "alliance":
        showAllianceScreen();
        break;
      case "market":
        showFleaMarketScreen();
        break;
      case "history":
        setCurrentScreen("history");
        render();
        break;
      case "status":
        showStatusScreen();
        break;
      case "inventory":
        setCurrentScreen("inventory");
        showInventoryScreen();
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
    
    if (action === currentScreen) {
      item.classList.add("is-active");
    }
  });
}

export function updateBottomMenu(): void {
  updateActiveState();
}
