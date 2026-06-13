import { USE_MOCK_STATE } from "./config";
import { applyInitialGameMode } from "./network/mode-switch";

import { ensureDevUnit } from "./game/formation";

import { initMapView, waitForMapContainerLayout, wakeMapView } from "./map-view";

import { ensureAuthSession, resetAuthSession } from "./network/auth-client";
import { ensureValidAuthSession, loadGameState, relogin } from "./network/session";

import { connect } from "./network/ws-client";

import { gameState, setConnectionStatus } from "./store";
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

async function startOnlineSession(
  options: BootstrapOptions & { prepareMapSession: () => Promise<void> },
): Promise<void> {
  let token = await ensureValidAuthSession();
  while (token) {
    if (await loadGameState(token)) {
      break;
    }
    resetAuthSession();
    options.render();
    token = await ensureAuthSession({ force: true });
  }
  if (!token) {
    options.render();
    return;
  }

  await options.prepareMapSession();
  connect({
    closeMenu: options.closeMenu,
    closeUnitSelect: options.closeUnitSelect,
  });
}

export async function bootstrapApp(options: BootstrapOptions): Promise<void> {
  await applyInitialGameMode();
  options.render();

  let mapReadyPromise: Promise<void> | null = null;
  const ensureMapReady = async () => {
    if (!mapReadyPromise) {
      mapReadyPromise = initMapView(options.mapContainer, {
        onTerritoryClick: options.onTerritoryClick,
      });
    }
    await mapReadyPromise;
  };

  const prepareMapSession = async () => {
    setConnectionStatus("online");
    options.render();
    await waitForMapContainerLayout(options.mapContainer);
    await ensureMapReady();
    wakeMapView(gameState);
  };

  const sessionOptions = { ...options, prepareMapSession };

  try {
    if (USE_MOCK_STATE) {
      ensureDevUnit();
      await prepareMapSession();
      return;
    }

    if (new URLSearchParams(location.search).has("relogin")) {
      await relogin(sessionOptions);
      return;
    }

    await startOnlineSession(sessionOptions);
  } catch (error) {
    console.error("bootstrap failed:", error);
    resetAuthSession();
    options.render();
    await ensureAuthSession({ force: true });
    options.render();
  }
}
