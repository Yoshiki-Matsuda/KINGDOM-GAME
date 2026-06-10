import {
  API_ORIGIN,
  DEV_AUTO_LOGIN,
  DEV_AUTO_LOGIN_PASSWORD,
  DEV_AUTO_LOGIN_USERNAME,
} from "../config";
import { showLoginModal } from "../ui/login-modal";
import { authToken, clearAuthSession, setAuthSession } from "../store";
import { resolvePlayerIdentity } from "./player-identity";

interface AuthResponse {
  token: string;
  player_id: string;
  username: string;
}

async function requestAuth(path: string, username: string, password: string): Promise<AuthResponse> {
  const response = await fetch(`${API_ORIGIN}${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? "認証に失敗しました");
  }
  return response.json() as Promise<AuthResponse>;
}

async function establishSession(token: string): Promise<string | null> {
  setAuthSession(token);
  const playerId = await resolvePlayerIdentity(token);
  return playerId ? token : null;
}

async function tryDevAutoLogin(): Promise<string | null> {
  if (!DEV_AUTO_LOGIN) return null;
  try {
    const result = await requestAuth(
      "/auth/login",
      DEV_AUTO_LOGIN_USERNAME,
      DEV_AUTO_LOGIN_PASSWORD,
    );
    return establishSession(result.token);
  } catch {
    return null;
  }
}

export async function ensureAuthSession(options?: { force?: boolean }): Promise<string | null> {
  if (options?.force) clearAuthSession();
  else if (authToken) {
    await resolvePlayerIdentity(authToken);
    return authToken;
  }

  if (!options?.force) {
    const devToken = await tryDevAutoLogin();
    if (devToken) return devToken;
  }

  let error: string | undefined;
  let lastUsername = "";

  while (true) {
    const form = await showLoginModal({ error, username: lastUsername });
    if (!form) return null;
    lastUsername = form.username;
    error = undefined;

    try {
      const path = form.register ? "/auth/register" : "/auth/login";
      const result = await requestAuth(path, form.username, form.password);
      const token = await establishSession(result.token);
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
