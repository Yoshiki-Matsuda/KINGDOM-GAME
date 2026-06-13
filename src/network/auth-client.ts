import {
  DEV_AUTO_LOGIN,
  DEV_AUTO_LOGIN_PASSWORD,
  DEV_AUTO_LOGIN_USERNAME,
} from "../config";
import type { GameMode } from "../config";
import { apiOriginForMode } from "../config";
import { showLoginModal } from "../ui/login-modal";
import { authToken, clearAuthSession, setAuthSession, setAuthenticatedPlayerId, setGameMode, setSelectedPvpServerId } from "../store";
import { resolvePlayerIdentity, peekPlayerIdentity } from "./player-identity";
import {
  discoverAvailableModes,
  preferReachableMode,
} from "./server-discovery";
import { getStoredGameMode, persistGameMode } from "./game-mode-storage";

interface AuthResponse {
  token: string;
  player_id: string;
  username: string;
}

/** 他モードの JWT をこのモード用トークンに交換（HUD 切替・起動時正規化） */
const EXCHANGE_TIMEOUT_MS = 5000;

export async function exchangeAuthToken(
  targetMode: GameMode,
  token: string,
): Promise<string | null> {
  try {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), EXCHANGE_TIMEOUT_MS);
    const response = await fetch(`${apiOriginForMode(targetMode)}/auth/exchange`, {
      method: "POST",
      headers: { authorization: `Bearer ${token}` },
      signal: controller.signal,
    });
    clearTimeout(timer);
    if (!response.ok) return null;
    const data = (await response.json()) as AuthResponse;
    return data.token ?? null;
  } catch {
    return null;
  }
}

export async function normalizeTokenForMode(
  targetMode: GameMode,
  token: string,
): Promise<string | null> {
  const exchanged = await exchangeAuthToken(targetMode, token);
  if (!exchanged) return null;
  const playerId = await peekPlayerIdentity(exchanged, targetMode);
  if (!playerId) return null;
  setAuthSession(exchanged);
  setAuthenticatedPlayerId(playerId);
  return exchanged;
}

async function requestAuthOnMode(
  mode: GameMode,
  path: string,
  username: string,
  password: string,
): Promise<AuthResponse> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), EXCHANGE_TIMEOUT_MS);
  let response: Response;
  try {
    response = await fetch(`${apiOriginForMode(mode)}${path}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ username, password }),
      signal: controller.signal,
    });
  } catch {
    clearTimeout(timer);
    throw new Error(
      `${mode.toUpperCase()} サーバーに接続できません（${apiOriginForMode(mode)}）。`,
    );
  }
  clearTimeout(timer);
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? "認証に失敗しました");
  }
  return response.json() as Promise<AuthResponse>;
}

async function establishSession(token: string, mode: GameMode): Promise<string | null> {
  const playerId = await peekPlayerIdentity(token, mode);
  if (!playerId) return null;
  setGameMode(mode);
  persistGameMode(mode);
  setAuthSession(token);
  setAuthenticatedPlayerId(playerId);
  return token;
}

function loginModeOrder(available: GameMode[]): GameMode[] {
  const stored = getStoredGameMode();
  const order: GameMode[] = [];
  if (available.includes(stored)) order.push(stored);
  for (const mode of available) {
    if (!order.includes(mode)) order.push(mode);
  }
  if (order.length === 0) {
    order.push(stored, stored === "pvp" ? "pve" : "pvp");
  }
  return order;
}

async function tryDevAutoLoginOnMode(mode: GameMode): Promise<string | null> {
  try {
    const result = await requestAuthOnMode(
      mode,
      "/auth/login",
      DEV_AUTO_LOGIN_USERNAME,
      DEV_AUTO_LOGIN_PASSWORD,
    );
    return establishSession(result.token, mode);
  } catch {
    return null;
  }
}

async function tryDevAutoLogin(): Promise<string | null> {
  if (!DEV_AUTO_LOGIN) return null;
  const available = await discoverAvailableModes();
  for (const mode of loginModeOrder(available)) {
    const token = await tryDevAutoLoginOnMode(mode);
    if (token) return token;
  }
  return null;
}

export async function ensureAuthSession(options?: { force?: boolean }): Promise<string | null> {
  if (options?.force) clearAuthSession();
  else if (authToken) {
    const playerId = await resolvePlayerIdentity(authToken);
    if (playerId) return authToken;
    clearAuthSession();
  }

  if (!options?.force) {
    const devToken = await tryDevAutoLogin();
    if (devToken) return devToken;
  }

  let error: string | undefined;
  let lastUsername = "";
  const available = await discoverAvailableModes();
  const defaultMode =
    available.length > 0
      ? await preferReachableMode(getStoredGameMode())
      : getStoredGameMode();

  while (true) {
    const form = await showLoginModal({
      error,
      username: lastUsername,
      mode: defaultMode,
      availableModes: available,
    });
    if (!form) return null;
    lastUsername = form.username;

    setGameMode(form.mode as GameMode);
    if (form.mode === "pvp" && form.pvpServerId) {
      setSelectedPvpServerId(form.pvpServerId);
    }

    try {
      const path = form.register ? "/auth/register" : "/auth/login";
      const result = await requestAuthOnMode(form.mode as GameMode, path, form.username, form.password);
      const token = await establishSession(result.token, form.mode as GameMode);
      if (token) return token;
      error = "プレイヤー情報の取得に失敗しました";
    } catch (caught) {
      error = caught instanceof Error ? caught.message : "認証に失敗しました";
    }
  }
}

export function resetAuthSession(): void {
  clearAuthSession();
}
