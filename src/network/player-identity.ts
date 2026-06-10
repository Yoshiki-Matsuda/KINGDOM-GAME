import { API_ORIGIN } from "../config";
import { setAuthenticatedPlayerId } from "../store";

interface WhoamiResponse {
  player_id: string;
  username: string;
}

/** サーバー /api/whoami が返すプレイヤーIDのみを信頼する */
export async function resolvePlayerIdentity(token: string): Promise<string | null> {
  const response = await fetch(`${API_ORIGIN}/api/whoami`, {
    headers: { authorization: `Bearer ${token}` },
  });
  if (!response.ok) return null;
  const data = (await response.json()) as WhoamiResponse;
  if (!data.player_id) return null;
  setAuthenticatedPlayerId(data.player_id);
  return data.player_id;
}
