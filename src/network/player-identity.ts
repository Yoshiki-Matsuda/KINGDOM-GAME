import type { GameMode } from "../config";
import { apiOriginForMode } from "../config";
import { gameMode, setAuthenticatedPlayerId } from "../store";

interface WhoamiResponse {
  player_id: string;
  username: string;
}

const WHOAMI_TIMEOUT_MS = 3000;

function originForMode(mode?: GameMode): string {
  return apiOriginForMode(mode ?? gameMode);
}

/** whoami を呼ぶだけ（トークン保存・プレイヤーID確定はしない） */
export async function peekPlayerIdentity(
  token: string,
  mode?: GameMode,
): Promise<string | null> {
  try {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), WHOAMI_TIMEOUT_MS);
    const response = await fetch(`${originForMode(mode)}/api/whoami`, {
      headers: { authorization: `Bearer ${token}` },
      signal: controller.signal,
    });
    clearTimeout(timer);
    if (!response.ok) return null;
    const data = (await response.json()) as WhoamiResponse;
    return data.player_id ?? null;
  } catch {
    return null;
  }
}

/** サーバー /api/whoami が返すプレイヤーIDのみを信頼する */
export async function resolvePlayerIdentity(token: string): Promise<string | null> {
  const playerId = await peekPlayerIdentity(token);
  if (!playerId) return null;
  setAuthenticatedPlayerId(playerId);
  return playerId;
}
