/**
 * キングダム戦略ゲーム — エントリポイント
 * DOM構築とモジュール接続のみ。ロジックは各モジュールに委譲。
 */

import "./style.css";
import { bootstrapApp, bindGlobalMenuDismiss } from "./app-bootstrap";
import { createAppElements } from "./app-dom";
import { createAppRenderer } from "./app-render";
import { setRenderCallback } from "./store";
import { closeMenu, showMenuAt } from "./ui/context-menu";
import { closeUnitSelect } from "./ui/unit-select";

const appEl = document.querySelector<HTMLDivElement>("#app")!;
const elements = createAppElements(appEl);
const render = createAppRenderer({
  homeEl: elements.homeEl,
  mapContainer: elements.mapContainer,
  logEl: elements.logEl,
  inventoryEl: elements.inventoryEl,
  marketEl: elements.marketEl,
  allianceEl: elements.allianceEl,
  packEl: elements.packEl,
  statusEl: elements.statusEl,
  rankingEl: elements.rankingEl,
});

setRenderCallback(render);
bindGlobalMenuDismiss(elements.menuEl, closeMenu);

(async () => {
  await bootstrapApp({
    mapContainer: elements.mapContainer,
    closeMenu,
    closeUnitSelect,
    onTerritoryClick: (territoryId, territory, screenX, screenY) => {
      showMenuAt(screenX, screenY, territoryId, territory);
    },
    render,
  });
})();
