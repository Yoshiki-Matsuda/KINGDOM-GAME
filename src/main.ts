/**
 * キングダム戦略ゲーム — エントリポイント
 * DOM構築とモジュール接続のみ。ロジックは各モジュールに委譲。
 */

import "./style.css";
import {
  USE_MOCK_STATE,
  gameState, setGameState,
  travelingUnits,
  currentScreen,
  setRenderCallback,
} from "./store";
import { ensureDevUnit } from "./game/formation";
import type { GameState } from "./store";
import { initMapView, updateMapView } from "./map-view";
import { connect } from "./network/ws-client";
import { createHudElement, renderHud } from "./ui/hud";
import { createLogElement, renderLog } from "./ui/log-panel";
import { createMenuElement, closeMenu, showMenuAt } from "./ui/context-menu";
import { createUnitSelectElement, closeUnitSelect, updateUnitSelectReturningList } from "./ui/unit-select";
import { createFormationElement } from "./ui/formation-screen";
import { createHomeElement } from "./ui/home-screen";
import { createBottomMenuElement, updateBottomMenu } from "./ui/bottom-menu";
import { createInventoryElement, renderInventory } from "./ui/inventory-screen";

// --- DOM構築 ---
const appEl = document.querySelector<HTMLDivElement>("#app")!;
const mapContainer = document.createElement("div");
mapContainer.className = "map-container";

const hudEl = createHudElement();
const logEl = createLogElement();
const menuEl = createMenuElement();
const unitSelectEl = createUnitSelectElement();
const formationEl = createFormationElement();
const homeEl = createHomeElement();
const inventoryEl = createInventoryElement();
const bottomMenuEl = createBottomMenuElement();

appEl.appendChild(mapContainer);
appEl.appendChild(hudEl);
appEl.appendChild(logEl);
appEl.appendChild(menuEl);
appEl.appendChild(unitSelectEl);
appEl.appendChild(formationEl);
appEl.appendChild(homeEl);
appEl.appendChild(inventoryEl);
appEl.appendChild(bottomMenuEl);

// --- レンダリング ---
function render(): void {
  const isHome = currentScreen === "home";
  const isHistory = currentScreen === "history";
  const isMap = currentScreen === "map";
  const isInventory = currentScreen === "inventory";
  
  homeEl.style.display = isHome ? "flex" : "none";
  mapContainer.style.display = isMap ? "block" : "none";
  logEl.style.display = isHistory ? "flex" : "none";
  inventoryEl.style.display = isInventory ? "flex" : "none";
  
  if (isInventory) {
    renderInventory();
  }
  
  renderHud();
  renderLog();
  updateBottomMenu();

  const travelingDestinations = (() => {
    if (travelingUnits.length === 0) return undefined;
    const byTarget = new Map<string, { secLeft: number; unitNames: string[] }>();
    const now = Date.now();
    for (const t of travelingUnits) {
      const secLeft = (t.arrivalTime - now) / 1000;
      if (secLeft <= 0) continue;
      const cur = byTarget.get(t.targetId);
      if (!cur) byTarget.set(t.targetId, { secLeft, unitNames: [t.unitName] });
      else {
        cur.unitNames.push(t.unitName);
        cur.secLeft = Math.min(cur.secLeft, secLeft);
      }
    }
    return Array.from(byTarget.entries()).map(([targetId, v]) => ({
      targetId,
      secLeft: v.secLeft,
      unitNames: v.unitNames,
    }));
  })();
  // @ts-ignore
  updateMapView(gameState, travelingDestinations);
  updateUnitSelectReturningList();
}

setRenderCallback(render);

// --- メニュー外クリックで閉じる ---
document.addEventListener("pointerdown", (e) => {
  if (!menuEl.contains(e.target as Node)) closeMenu();
});

// --- 起動 ---
(async () => {
  await initMapView(mapContainer, {
    onTerritoryClick: (territoryId, territory, screenX, screenY) => {
      showMenuAt(screenX, screenY, territoryId, territory);
    },
  });
  if (USE_MOCK_STATE) {
    ensureDevUnit();
  }
  render();
  if (!USE_MOCK_STATE) {
    try {
      const res = await fetch("http://127.0.0.1:3000/api/state");
      if (res.ok) {
        setGameState((await res.json()) as GameState);
        ensureDevUnit();
        render();
      }
    } catch {
      // サーバー未起動など
    }
    connect({ closeMenu, closeUnitSelect });
  }
})();
