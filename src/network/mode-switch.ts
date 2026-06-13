import type { GameMode } from "../config";
import { apiOriginForMode, stateUrlForMode, wsUrlForMode } from "../config";
import { getStoredGameMode, persistGameMode } from "./game-mode-storage";
import { exchangeAuthToken } from "./auth-client";
import { loadGameState } from "./session";
import {
  authToken,
  clearAuthSession,
  gameMode,
  setAuthSession,
  setConnectionStatus,
  setGameMode,
  setModeSwitchError,
  setWs,
  ws,
} from "../store";
import { connect } from "./ws-client";
import { isServerReachable, resolveInitialGameMode } from "./server-discovery";

export {
  discoverAvailableModes,
  isServerReachable,
  preferReachableMode,
  resolveInitialGameMode,
} from "./server-discovery";
export { getStoredGameMode, persistGameMode } from "./game-mode-storage";

/** 起動時: 保存モードまたは起動中サーバーに合わせて gameMode を設定 */
export async function applyInitialGameMode(): Promise<GameMode> {
  const stored = getStoredGameMode();
  const mode = await resolveInitialGameMode(stored);
  setGameMode(mode);
  return mode;
}

/** 切断 → トークン交換 → 再接続でモードを切り替える */
export async function switchGameMode(
  mode: GameMode,
  callbacks: { closeMenu: () => void; closeUnitSelect: () => void },
): Promise<boolean> {
  if (mode === gameMode) return true;

  if (!(await isServerReachable(mode))) {
    setModeSwitchError(`${mode.toUpperCase()} サーバーが起動していません`);
    return false;
  }

  setModeSwitchError(null);

  if (!authToken) {
    setGameMode(mode);
    persistGameMode(mode);
    return true;
  }

  if (ws) {
    ws.close();
    setWs(null);
  }

  const newToken = await exchangeAuthToken(mode, authToken);
  if (!newToken) {
    setModeSwitchError(`${mode.toUpperCase()} への切替に失敗しました`);
    return false;
  }

  setAuthSession(newToken);
  setGameMode(mode);

  const loaded = await loadGameState(newToken);
  if (!loaded) {
    setModeSwitchError("ワールドの読み込みに失敗しました");
    return false;
  }

  setConnectionStatus("online");
  connect(callbacks);
  return true;
}

export function currentApiOrigin(): string {
  return apiOriginForMode(gameMode);
}

export function currentStateUrl(): string {
  return stateUrlForMode(gameMode);
}

export function currentWsUrl(): string {
  return wsUrlForMode(gameMode);
}

export function disconnectForLogin(): void {
  if (ws) {
    ws.close();
    setWs(null);
  }
  clearAuthSession();
}
