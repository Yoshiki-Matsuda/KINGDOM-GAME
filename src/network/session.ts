/**
 * 認証セッションの検証・再ログイン
 */

import {
  DEV_AUTO_LOGIN,
  DEV_AUTO_LOGIN_USERNAME,
  STATE_URL,
} from "../config";
import { ensureDevUnit, validateFormedUnits } from "../game/formation";
import { ensureAuthSession, resetAuthSession } from "./auth-client";
import { resolvePlayerIdentity } from "./player-identity";
import { focusMapOnPlayerHome } from "../map-view";
import { connect } from "./ws-client";
import { authToken, setGameState } from "../store";
import type { GameState } from "../store";

export async function loadGameState(token: string): Promise<boolean> {
  try {
    const playerId = await resolvePlayerIdentity(token);
    if (!playerId) return false;

    const response = await fetch(STATE_URL, {
      headers: { authorization: `Bearer ${token}` },
    });
    if (!response.ok) return false;

    const state = (await response.json()) as GameState;
    if (!Array.isArray(state.territories) || state.territories.length === 0) {
      return false;
    }
    setGameState(state);
    validateFormedUnits();
    ensureDevUnit();
    return true;
  } catch {
    return false;
  }
}

/** 保存済みトークンを検証し、無効ならクリアしてログインを促す */
export async function ensureValidAuthSession(): Promise<string | null> {
  if (authToken) {
    const playerId = await resolvePlayerIdentity(authToken);
    const allowed = playerId && (!DEV_AUTO_LOGIN || playerId === DEV_AUTO_LOGIN_USERNAME);
    if (allowed) return authToken;
    resetAuthSession();
  }
  return ensureAuthSession();
}

type SessionCallbacks = {
  closeMenu: () => void;
  closeUnitSelect: () => void;
  render: () => void;
  ensureMapReady: () => Promise<void>;
};

export async function relogin(callbacks: SessionCallbacks): Promise<void> {
  resetAuthSession();
  const token = await ensureAuthSession({ force: true });
  if (!token) {
    callbacks.render();
    return;
  }
  await callbacks.ensureMapReady();
  const loaded = await loadGameState(token);
  if (!loaded) {
    resetAuthSession();
    callbacks.render();
    return;
  }
  callbacks.render();
  focusMapOnPlayerHome();
  connect({
    closeMenu: callbacks.closeMenu,
    closeUnitSelect: callbacks.closeUnitSelect,
  });
  callbacks.render();
  focusMapOnPlayerHome();
}
