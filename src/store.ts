/**
 * 中央ストア — Valtio proxy ベースのリアクティブ状態管理
 * 各モジュールはここから import して参照・更新する。循環依存を防ぐ。
 *
 * 設計:
 * - 全状態を Valtio proxy() に集約し、subscribe() で自動 render() を発火
 * - 既存コードとの互換性のため、export let + setter のインターフェースを維持
 * - 非リアクティブな値（ws, homeFacilities, renderCallbacks）は proxy 外で管理
 */

import { proxy, subscribe } from "valtio";
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

// --- 型定義 ---
export interface FormedUnit {
  id: string;
  name: string;
  indices: [number, number, number];
  monster_count: number;
  avgSpeed: number;
}

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

export type Screen = "map" | "home" | "history" | "inventory" | "market" | "alliance" | "pack" | "status" | "ranking";
export type FacilityType = string | null;
export type PendingUnitAction =
  | { type: "attack"; fromId: string; toId: string }
  | { type: "deploy"; territoryId: string }
  | { type: "explore"; fromId: string; toId: string }
  | null;

// --- Valtio proxy（全リアクティブ状態を集約） ---
const store = proxy({
  // ゲームモード
  gameMode: getStoredGameMode() as GameMode,
  // ゲーム状態
  gameState: (USE_MOCK_STATE ? getMockGameState() : DEFAULT_GAME_STATE) as GameState,
  // 接続状態
  connectionStatus: "offline" as "online" | "offline",
  // 認証
  authToken: storedAuthToken as string | null,
  // モード切替エラー
  modeSwitchError: null as string | null,
  // 編成
  formedUnitsList: [] as FormedUnit[],
  nextFormedUnitId: 1,
  formationSelected: [] as number[],
  bodyMonsterCounts: [] as number[],
  bodySpeeds: [] as number[],
  // 移動
  travelingUnits: [] as TravelingUnit[],
  nextTravelingId: 1,
  travelIntervalId: null as ReturnType<typeof setInterval> | null,
  // 画面状態
  currentScreen: "map" as Screen,
  // ユニット選択
  pendingUnitAction: null as PendingUnitAction,
});

// --- 非リアクティブ状態（proxy 外） ---
let authenticatedPlayerId: string | null = null;
let selectedPvpServerId: string | null = localStorage.getItem("kingdom.pvp_server_id");

/** WebSocket インスタンス（proxy 非対象 — ネイティブクラス） */
export let ws: WebSocket | null = null;
export function setWs(socket: WebSocket | null) { ws = socket; }

/** 本拠地施設マップ（proxy 非対象 — Map インスタンス） */
export const homeFacilities = new Map<string, FacilityType>();
export function setHomeFacility(col: number, row: number, type: FacilityType) {
  const key = `${col},${row}`;
  if (type) homeFacilities.set(key, type);
  else homeFacilities.delete(key);
}
export function getHomeFacility(col: number, row: number): FacilityType {
  return homeFacilities.get(`${col},${row}`) ?? null;
}

// --- レンダー コールバック ---
let renderCallback: (() => void) | null = null;
let renderMapSessionCallback: (() => void) | null = null;
export function setRenderCallback(cb: () => void) { renderCallback = cb; }
export function setRenderMapSessionCallback(cb: () => void) { renderMapSessionCallback = cb; }
export function render() { renderCallback?.(); }
/** WS tick 等: マップ表示中のみ HUD・地図・遠征オーバーレイを同期 */
export function renderMapSession() { renderMapSessionCallback?.(); }

// --- Valtio subscribe: proxy 状態変更時に自動で render() を発火 ---
subscribe(store, () => {
  render();
});

// --- ゲームモード ---
export let gameMode: GameMode = store.gameMode;
export function setGameMode(mode: GameMode) {
  gameMode = mode;
  store.gameMode = mode;
  persistGameMode(mode);
}

export function getGridCols(): number {
  return getWorldConfig(store.gameState).cols;
}

export function getGridRows(): number {
  return getWorldConfig(store.gameState).rows;
}

// --- ゲーム状態 ---
export let gameState: GameState = store.gameState;
export function setGameState(s: GameState) {
  Object.assign(store.gameState, s);
  gameState = store.gameState;
}

export let connectionStatus: "online" | "offline" = store.connectionStatus;
export function setConnectionStatus(s: "online" | "offline") {
  if (connectionStatus === s) return;
  connectionStatus = s;
  store.connectionStatus = s;
}

/** サーバー /api/whoami で確定したプレイヤーID（メモリのみ） */
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

export let authToken: string | null = store.authToken;
export function setAuthSession(token: string) {
  if (authToken === token && authenticatedPlayerId != null) return;
  authToken = token;
  store.authToken = token;
  authenticatedPlayerId = null;
  localStorage.setItem("kingdom.auth_token", token);
}
export function clearAuthSession() {
  authToken = null;
  store.authToken = null;
  authenticatedPlayerId = null;
  localStorage.removeItem("kingdom.auth_token");
  store.connectionStatus = "offline";
  connectionStatus = "offline";
}

const PVP_SERVER_STORAGE_KEY = "kingdom.pvp_server_id";
export function getSelectedPvpServerId(): string | null {
  return selectedPvpServerId;
}
export function setSelectedPvpServerId(serverId: string | null): void {
  selectedPvpServerId = serverId;
  if (serverId) localStorage.setItem(PVP_SERVER_STORAGE_KEY, serverId);
  else localStorage.removeItem(PVP_SERVER_STORAGE_KEY);
}

export let modeSwitchError: string | null = store.modeSwitchError;
export function setModeSwitchError(message: string | null) {
  modeSwitchError = message;
  store.modeSwitchError = message;
}

// --- 編成 ---
export let formedUnitsList: FormedUnit[] = store.formedUnitsList;
export function setFormedUnitsList(list: FormedUnit[]) {
  formedUnitsList = list;
  store.formedUnitsList = list;
}

export let nextFormedUnitId = store.nextFormedUnitId;
export function getNextFormedUnitId(): number {
  const id = ++store.nextFormedUnitId;
  nextFormedUnitId = id;
  return id;
}
export function setNextFormedUnitId(n: number): void {
  const val = Math.max(1, n);
  store.nextFormedUnitId = val;
  nextFormedUnitId = val;
}

export let formationSelected: number[] = store.formationSelected;
export function setFormationSelected(sel: number[]) {
  formationSelected = sel;
  store.formationSelected = sel;
}

export let bodyMonsterCounts: number[] = store.bodyMonsterCounts;
export function setBodyMonsterCounts(c: number[]) {
  bodyMonsterCounts = c;
  store.bodyMonsterCounts = c;
}

export let bodySpeeds: number[] = store.bodySpeeds;
export function setBodySpeeds(s: number[]) {
  bodySpeeds = s;
  store.bodySpeeds = s;
}

// --- 移動 ---
export let travelingUnits: TravelingUnit[] = store.travelingUnits;
export function setTravelingUnits(list: TravelingUnit[]) {
  travelingUnits = list;
  store.travelingUnits = list;
}

export let nextTravelingId = store.nextTravelingId;
export function getNextTravelingId(): number {
  const id = ++store.nextTravelingId;
  nextTravelingId = id;
  return id;
}

export let travelIntervalId: ReturnType<typeof setInterval> | null = store.travelIntervalId;
export function setTravelIntervalId(id: ReturnType<typeof setInterval> | null) {
  travelIntervalId = id;
  store.travelIntervalId = id;
}

// --- 画面状態 ---
export let currentScreen: Screen = store.currentScreen;
export function setCurrentScreen(s: Screen) {
  currentScreen = s;
  store.currentScreen = s;
}

// --- ユニット選択 ---
export let pendingUnitAction: PendingUnitAction = store.pendingUnitAction;
export function setPendingUnitAction(a: PendingUnitAction) {
  pendingUnitAction = a;
  store.pendingUnitAction = a;
}

// --- proxy 本体のエクスポート（新コード用） ---
export { store };

export type { GameState, Territory, SkillDataPayload, CardStatsPayload };
