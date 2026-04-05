import { STATE_URL, USE_MOCK_STATE } from "./config";
import { ensureDevUnit, validateFormedUnits } from "./game/formation";
import { initMapView } from "./map-view";
import { connect } from "./network/ws-client";
import { setGameState } from "./store";
import type { GameState, Territory } from "./store";

interface BootstrapOptions {
  mapContainer: HTMLDivElement;
  closeMenu: () => void;
  closeUnitSelect: () => void;
  onTerritoryClick: (territoryId: string, territory: Territory, screenX: number, screenY: number) => void;
  render: () => void;
}

export function bindGlobalMenuDismiss(menuEl: HTMLDivElement, closeMenu: () => void): void {
  document.addEventListener("pointerdown", (event) => {
    if (!menuEl.contains(event.target as Node)) {
      closeMenu();
    }
  });
}

export async function bootstrapApp(options: BootstrapOptions): Promise<void> {
  await initMapView(options.mapContainer, {
    onTerritoryClick: options.onTerritoryClick,
  });

  if (USE_MOCK_STATE) {
    ensureDevUnit();
    options.render();
    return;
  }

  options.render();

  try {
    const response = await fetch(STATE_URL);
    if (response.ok) {
      setGameState((await response.json()) as GameState);
      validateFormedUnits();
      ensureDevUnit();
      options.render();
    }
  } catch {
    // サーバー未起動など
  }

  connect({
    closeMenu: options.closeMenu,
    closeUnitSelect: options.closeUnitSelect,
  });
}
