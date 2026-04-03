import { currentScreen, gameState, travelingUnits } from "./store";
import { updateMapView, setMapVisible } from "./map-view";
import { renderHud } from "./ui/hud";
import { renderLog } from "./ui/log-panel";
import { updateBottomMenu } from "./ui/bottom-menu";
import { renderInventory } from "./ui/inventory-screen";
import { renderFleaMarket } from "./ui/flea-market-screen";
import { renderAlliance } from "./ui/alliance-screen";
import { renderPack } from "./ui/pack-screen";
import { renderStatus } from "./ui/status-screen";
import { renderRanking } from "./ui/ranking-screen";
import { updateUnitSelectReturningList } from "./ui/unit-select";

interface RenderElements {
  homeEl: HTMLElement;
  mapContainer: HTMLElement;
  logEl: HTMLElement;
  inventoryEl: HTMLElement;
  marketEl: HTMLElement;
  allianceEl: HTMLElement;
  packEl: HTMLElement;
  statusEl: HTMLElement;
  rankingEl: HTMLElement;
}

interface TravelingDestination {
  targetId: string;
  secLeft: number;
  unitNames: string[];
}

function getTravelingDestinations(now: number = Date.now()): TravelingDestination[] | undefined {
  if (travelingUnits.length === 0) return undefined;

  const byTarget = new Map<string, TravelingDestination>();
  for (const traveling of travelingUnits) {
    const secLeft = (traveling.arrivalTime - now) / 1000;
    if (secLeft <= 0) continue;

    const existing = byTarget.get(traveling.targetId);
    if (!existing) {
      byTarget.set(traveling.targetId, {
        targetId: traveling.targetId,
        secLeft,
        unitNames: [traveling.unitName],
      });
      continue;
    }

    existing.unitNames.push(traveling.unitName);
    existing.secLeft = Math.min(existing.secLeft, secLeft);
  }

  return Array.from(byTarget.values());
}

export function createAppRenderer(elements: RenderElements): () => void {
  return () => {
    const isHome = currentScreen === "home";
    const isHistory = currentScreen === "history";
    const isMap = currentScreen === "map";
    const isInventory = currentScreen === "inventory";
    const isMarket = currentScreen === "market";
    const isAlliance = currentScreen === "alliance";
    const isPack = currentScreen === "pack";
    const isStatus = currentScreen === "status";
    const isRanking = currentScreen === "ranking";

    elements.homeEl.style.display = isHome ? "flex" : "none";
    elements.mapContainer.style.display = isMap ? "block" : "none";
    elements.logEl.style.display = isHistory ? "flex" : "none";
    elements.inventoryEl.style.display = isInventory ? "flex" : "none";
    elements.marketEl.style.display = isMarket ? "flex" : "none";
    elements.allianceEl.style.display = isAlliance ? "flex" : "none";
    elements.packEl.style.display = isPack ? "flex" : "none";
    elements.statusEl.style.display = isStatus ? "flex" : "none";
    elements.rankingEl.style.display = isRanking ? "flex" : "none";

    if (isInventory) renderInventory();
    if (isMarket) renderFleaMarket();
    if (isAlliance) renderAlliance();
    if (isPack) renderPack();
    if (isStatus) renderStatus();
    if (isRanking) renderRanking();

    renderHud();
    renderLog();
    updateBottomMenu();
    setMapVisible(isMap);
    updateMapView(gameState, getTravelingDestinations());
    updateUnitSelectReturningList();
  };
}
