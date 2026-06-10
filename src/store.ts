/**
 * 中央ストア — 全ミュータブル状態を集約
 * 各モジュールはここから import して参照・更新する。循環依存を防ぐ。
 */

import { USE_MOCK_STATE } from "./config";
import type { GameState, Territory, SkillDataPayload, CardStatsPayload } from "./shared/game-state";
import { DEFAULT_GAME_STATE, DEFAULT_PLAYER_ID } from "./shared/game-state";
import { getMockGameState } from "./shared/mock-state";
export { USE_MOCK_STATE, WS_URL } from "./config";

// 旧バージョンの残骸。プレイヤーIDはサーバーが JWT から返す値のみを信頼する。
localStorage.removeItem("kingdom.player_id");

const storedAuthToken = localStorage.getItem("kingdom.auth_token");

// --- ゲーム状態 ---
export let gameState: GameState = USE_MOCK_STATE ? getMockGameState() : DEFAULT_GAME_STATE;
export function setGameState(s: GameState) { gameState = s; }

export let connectionStatus: "online" | "offline" = "offline";
export function setConnectionStatus(s: "online" | "offline") { connectionStatus = s; }

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
  authToken = token;
  authenticatedPlayerId = null;
  localStorage.setItem("kingdom.auth_token", token);
}
export function clearAuthSession() {
  authToken = null;
  authenticatedPlayerId = null;
  localStorage.removeItem("kingdom.auth_token");
}

export let attackSourceId: string | null = null;
export function setAttackSourceId(id: string | null) { attackSourceId = id; }

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
  | null;
export let pendingUnitAction: PendingUnitAction = null;
export function setPendingUnitAction(a: PendingUnitAction) { pendingUnitAction = a; }

// --- レンダー コールバック ---
let renderCallback: (() => void) | null = null;
export function setRenderCallback(cb: () => void) { renderCallback = cb; }
export function render() { renderCallback?.(); }

export type { GameState, Territory, SkillDataPayload, CardStatsPayload };
