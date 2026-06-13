/**
 * 中央ストア — 全ミュータブル状態を集約
 * 各モジュールはここから import して参照・更新する。循環依存を防ぐ。
 */

import { USE_MOCK_STATE, wsUrlForMode } from "./config";
import type { GameMode } from "./config";
import { getStoredGameMode, persistGameMode } from "./network/game-mode-storage";
import type { GameState, Territory, SkillDataPayload, CardStatsPayload } from "./shared/game-state";
import { DEFAULT_GAME_STATE, DEFAULT_PLAYER_ID, getWorldConfig } from "./shared/game-state";
import { getMockGameState } from "./shared/mock-state";

export { USE_MOCK_STATE } from "./config";
export function getWsUrl(): string {
  return wsUrlForMode(gameMode);
}

// 旧バージョンの残骸。プレイヤーIDはサーバーが JWT から返す値のみを信頼する。
localStorage.removeItem("kingdom.player_id");

const storedAuthToken = localStorage.getItem("kingdom.auth_token");

// --- ゲームモード ---
export let gameMode: GameMode = getStoredGameMode();
export function setGameMode(mode: GameMode) {
  gameMode = mode;
  persistGameMode(mode);
}

export function getGridCols(): number {
  return getWorldConfig(gameState).cols;
}

export function getGridRows(): number {
  return getWorldConfig(gameState).rows;
}

// --- ゲーム状態 ---
export let gameState: GameState = USE_MOCK_STATE ? getMockGameState() : DEFAULT_GAME_STATE;
export function setGameState(s: GameState) { gameState = s; }

export let connectionStatus: "online" | "offline" = "offline";
export function setConnectionStatus(s: "online" | "offline") {
  if (connectionStatus === s) return;
  connectionStatus = s;
  render();
}

/** サーバー /api/whoami で確定したプレイヤーID（メモリのみ） */
let authenticatedPlayerId: string | null = null;
export function setAuthenticatedPlayerId(playerId: string | null) {
  if (authenticatedPlayerId === playerId) return;
  authenticatedPlayerId = playerId;
  render();
}

/** /api/whoami でプレイヤーIDが確定済みか */
export function isPlayerIdentityResolved(): boolean {
  return USE_MOCK_STATE || authenticatedPlayerId != null;
}

/** 操作中プレイヤーID（未ログイン時のみ DEFAULT_PLAYER_ID） */
export function getLocalPlayerId(): string {
  if (USE_MOCK_STATE) return DEFAULT_PLAYER_ID;
  if (authenticatedPlayerId) return authenticatedPlayerId;
  return DEFAULT_PLAYER_ID;
}

export let authToken: string | null = storedAuthToken;
export function setAuthSession(token: string) {
  if (authToken === token && authenticatedPlayerId != null) return;
  authToken = token;
  authenticatedPlayerId = null;
  localStorage.setItem("kingdom.auth_token", token);
}
export function clearAuthSession() {
  authToken = null;
  authenticatedPlayerId = null;
  localStorage.removeItem("kingdom.auth_token");
  setConnectionStatus("offline");
}

const PVP_SERVER_STORAGE_KEY = "kingdom.pvp_server_id";
let selectedPvpServerId: string | null = localStorage.getItem(PVP_SERVER_STORAGE_KEY);

export function getSelectedPvpServerId(): string | null {
  return selectedPvpServerId;
}

export function setSelectedPvpServerId(serverId: string | null): void {
  selectedPvpServerId = serverId;
  if (serverId) localStorage.setItem(PVP_SERVER_STORAGE_KEY, serverId);
  else localStorage.removeItem(PVP_SERVER_STORAGE_KEY);
}

export let modeSwitchError: string | null = null;
export function setModeSwitchError(message: string | null) {
  modeSwitchError = message;
  render();
}

// --- WebSocket ---
export let ws: WebSocket | null = null;
export function setWs(socket: WebSocket | null) { ws = socket; }

// --- 編成 ---
export interface FormedUnit {
  id: string;
  name: string;
  indices: [number, number, number];
  monster_count: number;
  avgSpeed: number;
}
export let formedUnitsList: FormedUnit[] = [];
export function setFormedUnitsList(list: FormedUnit[]) { formedUnitsList = list; }

export let nextFormedUnitId = 1;
export function getNextFormedUnitId(): number { return nextFormedUnitId++; }
export function setNextFormedUnitId(n: number): void { nextFormedUnitId = Math.max(1, n); }

export let formationSelected: number[] = [];
export function setFormationSelected(sel: number[]) { formationSelected = sel; }

export let bodyMonsterCounts: number[] = [];
export function setBodyMonsterCounts(c: number[]) { bodyMonsterCounts = c; }

export let bodySpeeds: number[] = [];
export function setBodySpeeds(s: number[]) { bodySpeeds = s; }

// --- 移動 ---
export interface TravelingUnit {
  id: string;
  unitId: string;
  unitName: string;
  actionType: "attack" | "deploy" | "return";
  targetId: string;
  fromId?: string;
  count: number;
  monstersPerBody: number[];
  speedPerBody: number[];
  bodyNames: string[];
  skillsPerBody: SkillDataPayload[];
  statsPerBody: CardStatsPayload[];
  ownedCardIndices?: number[];
  departureTime: number;
  arrivalTime: number;
}
export let travelingUnits: TravelingUnit[] = [];
export function setTravelingUnits(list: TravelingUnit[]) { travelingUnits = list; }

export let nextTravelingId = 1;
export function getNextTravelingId(): number { return nextTravelingId++; }

export let travelIntervalId: ReturnType<typeof setInterval> | null = null;
export function setTravelIntervalId(id: ReturnType<typeof setInterval> | null) { travelIntervalId = id; }

// --- 画面状態 ---
export type Screen = "map" | "home" | "history" | "inventory" | "market" | "alliance" | "pack" | "status" | "ranking";
export let currentScreen: Screen = "map";
export function setCurrentScreen(s: Screen) { currentScreen = s; }

// --- 本拠地施設 ---
export type FacilityType = string | null;
export const homeFacilities = new Map<string, FacilityType>();
export function setHomeFacility(col: number, row: number, type: FacilityType) {
  const key = `${col},${row}`;
  if (type) homeFacilities.set(key, type);
  else homeFacilities.delete(key);
}
export function getHomeFacility(col: number, row: number): FacilityType {
  return homeFacilities.get(`${col},${row}`) ?? null;
}

// --- ユニット選択 ---
export type PendingUnitAction =
  | { type: "attack"; fromId: string; toId: string }
  | { type: "deploy"; territoryId: string }
  | { type: "explore"; fromId: string; toId: string }
  | null;
export let pendingUnitAction: PendingUnitAction = null;
export function setPendingUnitAction(a: PendingUnitAction) { pendingUnitAction = a; }

// --- レンダー コールバック ---
let renderCallback: (() => void) | null = null;
let renderMapSessionCallback: (() => void) | null = null;
export function setRenderCallback(cb: () => void) { renderCallback = cb; }
export function setRenderMapSessionCallback(cb: () => void) { renderMapSessionCallback = cb; }
export function render() { renderCallback?.(); }
/** WS tick 等: マップ表示中のみ HUD・地図・遠征オーバーレイを同期 */
export function renderMapSession() { renderMapSessionCallback?.(); }

export type { GameState, Territory, SkillDataPayload, CardStatsPayload };
