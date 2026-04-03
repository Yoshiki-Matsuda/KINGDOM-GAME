import { createHudElement } from "./ui/hud";
import { createLogElement } from "./ui/log-panel";
import { createMenuElement } from "./ui/context-menu";
import { createUnitSelectElement } from "./ui/unit-select";
import { createFormationElement } from "./ui/formation-screen";
import { createHomeElement } from "./ui/home-screen";
import { createBottomMenuElement } from "./ui/bottom-menu";
import { createInventoryElement } from "./ui/inventory-screen";
import { createFleaMarketElement } from "./ui/flea-market-screen";
import { createAllianceElement } from "./ui/alliance-screen";
import { createPackElement } from "./ui/pack-screen";
import { createStatusElement } from "./ui/status-screen";
import { createRankingElement } from "./ui/ranking-screen";

export interface AppElements {
  mapContainer: HTMLDivElement;
  hudEl: HTMLDivElement;
  logEl: HTMLDivElement;
  menuEl: HTMLDivElement;
  unitSelectEl: HTMLDivElement;
  formationEl: HTMLDivElement;
  homeEl: HTMLDivElement;
  inventoryEl: HTMLDivElement;
  marketEl: HTMLDivElement;
  allianceEl: HTMLDivElement;
  packEl: HTMLDivElement;
  statusEl: HTMLDivElement;
  rankingEl: HTMLDivElement;
  bottomMenuEl: HTMLDivElement;
}

export function createAppElements(appEl: HTMLDivElement): AppElements {
  const mapContainer = document.createElement("div");
  mapContainer.className = "map-container";

  const elements: AppElements = {
    mapContainer,
    hudEl: createHudElement(),
    logEl: createLogElement(),
    menuEl: createMenuElement(),
    unitSelectEl: createUnitSelectElement(),
    formationEl: createFormationElement(),
    homeEl: createHomeElement(),
    inventoryEl: createInventoryElement(),
    marketEl: createFleaMarketElement(),
    allianceEl: createAllianceElement(),
    packEl: createPackElement(),
    statusEl: createStatusElement(),
    rankingEl: createRankingElement(),
    bottomMenuEl: createBottomMenuElement(),
  };

  appEl.appendChild(elements.mapContainer);
  appEl.appendChild(elements.hudEl);
  appEl.appendChild(elements.logEl);
  appEl.appendChild(elements.menuEl);
  appEl.appendChild(elements.unitSelectEl);
  appEl.appendChild(elements.formationEl);
  appEl.appendChild(elements.homeEl);
  appEl.appendChild(elements.inventoryEl);
  appEl.appendChild(elements.marketEl);
  appEl.appendChild(elements.allianceEl);
  appEl.appendChild(elements.packEl);
  appEl.appendChild(elements.statusEl);
  appEl.appendChild(elements.rankingEl);
  appEl.appendChild(elements.bottomMenuEl);

  return elements;
}
