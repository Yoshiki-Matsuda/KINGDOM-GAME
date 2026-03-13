import { currentScreen, gameState, travelingUnits } from "./store";
import { updateMapView } from "./map-view";
import { renderHud } from "./ui/hud";
import { renderLog } from "./ui/log-panel";
import { updateBottomMenu } from "./ui/bottom-menu";
import { renderInventory } from "./ui/inventory-screen";
import { updateUnitSelectReturningList } from "./ui/unit-select";

interface RenderElements {
  homeEl: HTMLElement;
  mapContainer: HTMLElement;
  logEl: HTMLElement;
  inventoryEl: HTMLElement;
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

    elements.homeEl.style.display = isHome ? "flex" : "none";
    elements.mapContainer.style.display = isMap ? "block" : "none";
    elements.logEl.style.display = isHistory ? "flex" : "none";
    elements.inventoryEl.style.display = isInventory ? "flex" : "none";

    if (isInventory) {
      renderInventory();
    }

    renderHud();
    renderLog();
    updateBottomMenu();
    updateMapView(gameState, getTravelingDestinations());
    updateUnitSelectReturningList();
  };
}
