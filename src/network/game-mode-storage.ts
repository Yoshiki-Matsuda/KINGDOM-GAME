import type { GameMode } from "../config";

const MODE_STORAGE_KEY = "kingdom.game_mode";

export function getStoredGameMode(): GameMode {
  const stored = localStorage.getItem(MODE_STORAGE_KEY);
  return stored === "pve" ? "pve" : "pvp";
}

export function persistGameMode(mode: GameMode): void {
  localStorage.setItem(MODE_STORAGE_KEY, mode);
}
