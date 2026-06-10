import { USE_MOCK_STATE } from "./config";

import { ensureDevUnit } from "./game/formation";

import { focusMapOnPlayerHome, initMapView } from "./map-view";

import { resetAuthSession } from "./network/auth-client";
import { ensureValidAuthSession, loadGameState, relogin } from "./network/session";

import { connect } from "./network/ws-client";

import { authToken } from "./store";

import type { Territory } from "./store";



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

  let mapReadyPromise: Promise<void> | null = null;

  const ensureMapReady = async () => {

    if (!mapReadyPromise) {

      mapReadyPromise = initMapView(options.mapContainer, {

        onTerritoryClick: options.onTerritoryClick,

      });

    }

    await mapReadyPromise;

  };



  const sessionOptions = { ...options, ensureMapReady };



  if (USE_MOCK_STATE) {

    await ensureMapReady();

    ensureDevUnit();

    options.render();
    focusMapOnPlayerHome();

    return;

  }



  if (new URLSearchParams(location.search).has("relogin")) {

    await relogin(sessionOptions);
    focusMapOnPlayerHome();
    return;

  }



  const token = await ensureValidAuthSession();

  if (!token) {

    options.render();

    return;

  }



  await ensureMapReady();

  const loaded = await loadGameState(token);
  if (!loaded) {
    resetAuthSession();
    options.render();
    return;
  }

  options.render();
  focusMapOnPlayerHome();

  if (!authToken) {

    options.render();

    return;

  }



  connect({

    closeMenu: options.closeMenu,

    closeUnitSelect: options.closeUnitSelect,

  });

}


