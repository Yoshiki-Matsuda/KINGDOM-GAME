import {
  USE_MOCK_STATE, authToken, currentScreen, gameState,
  isPlayerIdentityResolved, getLocalPlayerId,
} from "./store";
import { updateMapView, setMapVisible, wakeMapView, type TravelingDestinationOverlay } from "./map-view";
import { buildMarchMapOverlays } from "./game/march-overlays";
import { renderHud } from "./ui/hud";
import { renderLog } from "./ui/log-panel";
import { updateHudSettings } from "./ui/hud-settings";
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

function getTravelingDestinations(now: number = Date.now()): TravelingDestinationOverlay[] {
  return buildMarchMapOverlays(gameState, getLocalPlayerId(), now);
}

let mapWasVisible = false;

/** マップの表示/非表示と wake/update を統一制御 */
function applyMapVisibility(canShowMap: boolean): void {
  setMapVisible(canShowMap);
  if (canShowMap) {
    if (!mapWasVisible) {
      wakeMapView(gameState, getTravelingDestinations());
    } else {
      updateMapView(gameState, getTravelingDestinations());
    }
  }
  mapWasVisible = canShowMap;
}

export function createAppRenderer(elements: RenderElements): {
  render: () => void;
  renderMapSession: () => void;
} {
  const appEl = document.querySelector<HTMLDivElement>("#app");

  const renderMapSession = () => {
    if (currentScreen !== "map") return;

    const sessionReady = USE_MOCK_STATE || (authToken != null && isPlayerIdentityResolved());
    const canShowMap = sessionReady;

    updateHudSettings();
    updateBottomMenu();
    applyMapVisibility(canShowMap);
    updateUnitSelectReturningList();
  };

  const render = () => {
    const sessionReady = USE_MOCK_STATE || (authToken != null && isPlayerIdentityResolved());
    const isHome = currentScreen === "home";
    const isHistory = currentScreen === "history";
    const isMap = currentScreen === "map";
    const isInventory = currentScreen === "inventory";
    const isMarket = currentScreen === "market";
    const isAlliance = currentScreen === "alliance";
    const isPack = currentScreen === "pack";
    const isStatus = currentScreen === "status";
    const isRanking = currentScreen === "ranking";

    appEl?.setAttribute("data-screen", currentScreen);

    const showHomeBackdrop = !USE_MOCK_STATE && !sessionReady;
    elements.homeEl.style.display = isHome || showHomeBackdrop ? "flex" : "none";
    const canShowMap = isMap && sessionReady;
    elements.mapContainer.style.display = canShowMap ? "block" : "none";
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
    updateHudSettings();
    updateBottomMenu();
    applyMapVisibility(canShowMap);
    updateUnitSelectReturningList();
  };

  return { render, renderMapSession };
}
