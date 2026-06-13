/**
 * WebSocket接続管理
 */

import {
  ws, setWs,
  setConnectionStatus, setGameState,
  renderMapSession, getWsUrl, authToken, currentScreen,
} from "../store";
import type { GameState } from "../store";
import { ensureDevUnit, validateFormedUnits } from "../game/formation";
import {
  flushPendingFormedUnitsToServer,
  hydrateFormedUnitsFromGameState,
} from "../game/formed-units-persist";
import { refreshFormationScreenIfOpen } from "../ui/formation-screen";
import { renderHud } from "../ui/hud";
import { resetAuthSession } from "./auth-client";

let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let activeCallbacks: {
  closeMenu: () => void;
  closeUnitSelect: () => void;
} | null = null;
/** 古いソケットの onclose が新しい接続状態を上書きしないよう世代で無効化する */
let wsGeneration = 0;

function clearReconnectTimer(): void {
  if (reconnectTimer !== null) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
}

function detachSocketHandlers(socket: WebSocket): void {
  socket.onopen = null;
  socket.onmessage = null;
  socket.onerror = null;
  socket.onclose = null;
}

function scheduleReconnect(): void {
  if (!authToken || !activeCallbacks) return;
  clearReconnectTimer();
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    if (authToken && activeCallbacks) {
      connect(activeCallbacks);
    }
  }, 3000);
}

function applyServerState(
  message: GameState,
  _callbacks: { closeMenu: () => void; closeUnitSelect: () => void },
): void {
  setConnectionStatus("online");
  setGameState(message);

  const flushedPending = flushPendingFormedUnitsToServer();
  if (!flushedPending) {
    hydrateFormedUnitsFromGameState();
  }

  validateFormedUnits();
  ensureDevUnit();

  // ヘッダー資源は独立コンポーネント — 表示中のマップ HUD を tick 同期（非表示時は DOM のみ更新）
  renderHud();

  if (currentScreen === "map") {
    refreshFormationScreenIfOpen();
    renderMapSession();
  }
}

/** WebSocket接続を開始する。UIコールバックを受け取ってメニュー・ユニット選択を閉じる */
export function connect(callbacks: {
  closeMenu: () => void;
  closeUnitSelect: () => void;
}): void {
  activeCallbacks = callbacks;
  clearReconnectTimer();

  if (ws?.readyState === WebSocket.OPEN) {
    setConnectionStatus("online");
    return;
  }
  if (!authToken) return;

  if (ws && ws.readyState !== WebSocket.CLOSED) {
    const stale = ws;
    detachSocketHandlers(stale);
    stale.close();
  }

  const tokenForAuth = authToken;
  const generation = ++wsGeneration;
  const socket = new WebSocket(getWsUrl());
  setWs(socket);

  socket.onopen = () => {
    if (generation !== wsGeneration) return;
    socket.send(JSON.stringify({ type: "auth", token: tokenForAuth }));
  };

  socket.onmessage = (e) => {
    if (generation !== wsGeneration) return;
    try {
      const message = JSON.parse(e.data);
      if (message.error) {
        console.warn("Server message:", message);
        if (message.error === "auth_invalid" || message.error === "auth_required") {
          resetAuthSession();
        }
        setConnectionStatus("offline");
        return;
      }
      if (!Array.isArray(message.territories)) {
        console.warn("Server message:", message);
        return;
      }
      applyServerState(message as GameState, callbacks);
    } catch {
      console.warn("Invalid state:", e.data);
    }
  };

  socket.onclose = () => {
    if (generation !== wsGeneration) return;
    setConnectionStatus("offline");
    scheduleReconnect();
  };

  socket.onerror = () => {
    if (generation !== wsGeneration) return;
    setConnectionStatus("offline");
  };
}
