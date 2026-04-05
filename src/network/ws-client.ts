/**
 * WebSocket接続管理
 */

import {
  ws, setWs,
  setConnectionStatus, setGameState,
  render, WS_URL,
} from "../store";
import type { GameState } from "../store";
import { ensureDevUnit, validateFormedUnits } from "../game/formation";

/** WebSocket接続を開始する。UIコールバックを受け取ってメニュー・ユニット選択を閉じる */
export function connect(callbacks: {
  closeMenu: () => void;
  closeUnitSelect: () => void;
}): void {
  if (ws?.readyState === WebSocket.OPEN) return;
  setConnectionStatus("offline");
  render();

  const socket = new WebSocket(WS_URL);
  setWs(socket);

  socket.onopen = () => {
    setConnectionStatus("online");
    render();
  };

  socket.onmessage = (e) => {
    try {
      setGameState(JSON.parse(e.data) as GameState);
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
  };

  socket.onerror = () => {
    setConnectionStatus("offline");
    render();
  };
}
