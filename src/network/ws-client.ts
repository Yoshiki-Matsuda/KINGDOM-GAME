/**
 * WebSocket接続管理
 */

import {
  ws, setWs,
  setConnectionStatus, setGameState,
  render, WS_URL, authToken,
} from "../store";
import type { GameState } from "../store";
import { ensureDevUnit, validateFormedUnits } from "../game/formation";
import { resetAuthSession } from "./auth-client";

let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let activeCallbacks: {
  closeMenu: () => void;
  closeUnitSelect: () => void;
} | null = null;

function clearReconnectTimer(): void {
  if (reconnectTimer !== null) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
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

/** WebSocket接続を開始する。UIコールバックを受け取ってメニュー・ユニット選択を閉じる */
export function connect(callbacks: {
  closeMenu: () => void;
  closeUnitSelect: () => void;
}): void {
  activeCallbacks = callbacks;
  clearReconnectTimer();

  if (ws?.readyState === WebSocket.OPEN) return;
  if (!authToken) return;

  if (ws && ws.readyState !== WebSocket.CLOSED) {
    ws.close();
  }

  setConnectionStatus("offline");
  render();

  const socket = new WebSocket(WS_URL);
  setWs(socket);

  socket.onopen = () => {
    socket.send(JSON.stringify({ type: "auth", token: authToken }));
  };

  socket.onmessage = (e) => {
    try {
      const message = JSON.parse(e.data);
      if (message.error) {
        console.warn("Server message:", message);
        if (message.error === "auth_invalid" || message.error === "auth_required") {
          resetAuthSession();
        }
        setConnectionStatus("offline");
        render();
        return;
      }
      if (!Array.isArray(message.territories)) {
        console.warn("Server message:", message);
        return;
      }
      setConnectionStatus("online");
      setGameState(message as GameState);
      validateFormedUnits();
      ensureDevUnit();
      callbacks.closeMenu();
      callbacks.closeUnitSelect();
      render();
    } catch {
      console.warn("Invalid state:", e.data);
    }
  };

  socket.onclose = () => {
    setConnectionStatus("offline");
    render();
    scheduleReconnect();
  };

  socket.onerror = () => {
    setConnectionStatus("offline");
    render();
  };
}
