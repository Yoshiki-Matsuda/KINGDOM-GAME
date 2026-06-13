/**
 * 認証セッションの検証・再ログイン
 */

import {
  DEV_AUTO_LOGIN,
  DEV_AUTO_LOGIN_USERNAME,
} from "../config";
import { currentStateUrl, disconnectForLogin, persistGameMode } from "./mode-switch";
import { discoverAvailableModes } from "./server-discovery";
import { ensureDevUnit, validateFormedUnits } from "../game/formation";
import { hydrateFormedUnitsFromGameState } from "../game/formed-units-persist";
import {
  ensureAuthSession,
  normalizeTokenForMode,
  resetAuthSession,
} from "./auth-client";
import { peekPlayerIdentity, resolvePlayerIdentity } from "./player-identity";
import { connect } from "./ws-client";
import { authToken, gameMode, setAuthenticatedPlayerId, setGameMode, setGameState } from "../store";
import type { GameState } from "../store";

const STATE_LOAD_TIMEOUT_MS = 10000;

export async function loadGameState(token: string): Promise<boolean> {
  try {
    const playerId = await peekPlayerIdentity(token);
    if (!playerId) return false;

    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), STATE_LOAD_TIMEOUT_MS);
    const response = await fetch(currentStateUrl(), {
      headers: { authorization: `Bearer ${token}` },
      signal: controller.signal,
    });
    clearTimeout(timer);
    if (!response.ok) return false;

    const state = (await response.json()) as GameState;
    if (!Array.isArray(state.territories) || state.territories.length === 0) {
      return false;
    }
    setAuthenticatedPlayerId(playerId);
    setGameState(state);
    hydrateFormedUnitsFromGameState();
    validateFormedUnits();
    ensureDevUnit();
    return true;
  } catch {
    return false;
  }
}

/** 保存済みトークンを検証し、無効ならクリアしてログインを促す */
export async function ensureValidAuthSession(): Promise<string | null> {
  if (!authToken) {
    return ensureAuthSession();
  }

  let playerId = await resolvePlayerIdentity(authToken);
  if (playerId) {
    const allowed = !DEV_AUTO_LOGIN || playerId === DEV_AUTO_LOGIN_USERNAME;
    if (allowed) return authToken;
  }

  const normalized = await normalizeTokenForMode(gameMode, authToken);
  if (normalized) {
    playerId = await resolvePlayerIdentity(normalized);
    if (playerId && (!DEV_AUTO_LOGIN || playerId === DEV_AUTO_LOGIN_USERNAME)) {
      return normalized;
    }
  }

  const available = await discoverAvailableModes();
  for (const mode of available) {
    if (mode === gameMode) continue;
    const token = await normalizeTokenForMode(mode, authToken);
    if (!token) continue;
    setGameMode(mode);
    persistGameMode(mode);
    playerId = await resolvePlayerIdentity(token);
    if (playerId && (!DEV_AUTO_LOGIN || playerId === DEV_AUTO_LOGIN_USERNAME)) {
      return token;
    }
  }

  resetAuthSession();
  return ensureAuthSession({ force: true });
}

type SessionCallbacks = {
  closeMenu: () => void;
  closeUnitSelect: () => void;
  render: () => void;
  prepareMapSession: () => Promise<void>;
};

export async function relogin(callbacks: SessionCallbacks): Promise<void> {
  resetAuthSession();
  let token = await ensureAuthSession({ force: true });
  while (token) {
    if (await loadGameState(token)) {
      break;
    }
    resetAuthSession();
    callbacks.render();
    token = await ensureAuthSession({ force: true });
  }
  if (!token) {
    callbacks.render();
    return;
  }
  await callbacks.prepareMapSession();
  connect({
    closeMenu: callbacks.closeMenu,
    closeUnitSelect: callbacks.closeUnitSelect,
  });
  callbacks.render();
}

/** ログアウト → ログインモーダル → 再セッション確立 */
export async function logout(callbacks: Omit<SessionCallbacks, "prepareMapSession">): Promise<void> {
  disconnectForLogin();
  callbacks.render();
  let token = await ensureAuthSession({ force: true });
  while (token) {
    if (await loadGameState(token)) {
      break;
    }
    resetAuthSession();
    callbacks.render();
    token = await ensureAuthSession({ force: true });
  }
  if (!token) {
    callbacks.render();
    return;
  }
  connect({
    closeMenu: callbacks.closeMenu,
    closeUnitSelect: callbacks.closeUnitSelect,
  });
  callbacks.render();
}
