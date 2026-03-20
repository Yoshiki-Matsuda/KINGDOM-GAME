/**
 * ゲーム状態の共通型（サーバーと同一構造）
 * 最終形 PvPvE を想定。
 */

/** 遺跡の難易度 */
export type RuinDifficulty = "easy" | "normal" | "hard" | "extreme";

/** 遺跡情報 */
export interface RuinInfo {
  /** 遺跡のフォーメーション名 */
  formation_name: string;
  /** 難易度 */
  difficulty: RuinDifficulty;
  /** 敵タイプ（3体） */
  enemies: string[];
  /** 敵の表示名（3体） */
  enemy_names?: string[];
  /** 敵エナジー（3体） */
  enemy_energies: number[];
  /** 消滅時刻（Unix timestamp ms） */
  expires_at?: number;
}

/** レベル 1=平原 〜 6=川。地形見た目・PvE 敵の強さと連動 */
export interface Territory {
  id: string;
  name: string;
  level: number;
  owner_id?: string | null;
  troops: number;
  /** 体ごとのエナジー。戦闘はこの順で1体ずつ行う */
  body_energies?: number[] | null;
  /** 体ごとの表示名（戦闘ログ用） */
  body_names?: string[] | null;
  /** 遺跡情報（存在する場合） */
  ruin?: RuinInfo | null;
}

/** インベントリ内のアイテム */
export interface InventoryItem {
  item_id: string;
  count: number;
}

/** 建設済み施設 */
export interface BuiltFacility {
  facility_id: string;
  level: number;
  /** 建設完了時刻（Unix timestamp ms）。nullなら完了済み */
  build_complete_at?: number | null;
  /** 配置座標（ホームマップ上） */
  position?: { col: number; row: number };
}

/** デフォルトのプレイヤーID（シングルプレイ時） */
export const DEFAULT_PLAYER_ID = "player";

/** プレイヤー固有のデータ */
export interface PlayerData {
  /** プレイヤーID */
  player_id: string;
  /** 本拠地の領地ID */
  home_territory_id: string;
  /** インベントリ */
  inventory: InventoryItem[];
  /** 建設済み施設一覧 */
  facilities: BuiltFacility[];
  /** 所持カード（カードID） */
  owned_cards: number[];
  /** 援軍を送れる他プレイヤーのID（クラン・配下など） */
  allied_player_ids: string[];
}

export interface GameState {
  turn: number;
  phase: string;
  territories: Territory[];
  /** バックエンドで発生した行動ログ。ユーザーは閲覧のみ */
  log: string[];
  /** 全プレイヤーのデータ（プレイヤーID -> PlayerData） */
  players?: Record<string, PlayerData>;
  
  // === 後方互換性のため残す（シングルプレイ時はここに直接入る） ===
  /** 援軍を送れる領の owner_id（自領 "player" に加え、クラン・配下など）。空なら自領のみ */
  deployable_owner_ids?: string[];
  /** プレイヤーのインベントリ（シングルプレイ用、マルチでは players を参照） */
  inventory?: InventoryItem[];
  /** 建設済み施設一覧（シングルプレイ用） */
  facilities?: BuiltFacility[];
  /** プレイヤーの所持カード（シングルプレイ用） */
  owned_cards?: number[];
}

/** プレイヤーのインベントリを取得（後方互換対応） */
export function getPlayerInventory(state: GameState, playerId: string = DEFAULT_PLAYER_ID): InventoryItem[] {
  return state.players?.[playerId]?.inventory ?? state.inventory ?? [];
}

/** プレイヤーの施設を取得 */
export function getPlayerFacilities(state: GameState, playerId: string = DEFAULT_PLAYER_ID): BuiltFacility[] {
  return state.players?.[playerId]?.facilities ?? state.facilities ?? [];
}

/** プレイヤーの所持カードを取得 */
export function getPlayerOwnedCards(state: GameState, playerId: string = DEFAULT_PLAYER_ID): number[] {
  return state.players?.[playerId]?.owned_cards ?? state.owned_cards ?? [];
}

/** プレイヤーが援軍を送れるowner_idリストを取得 */
export function getDeployableOwnerIds(state: GameState, playerId: string = DEFAULT_PLAYER_ID): string[] {
  const playerData = state.players?.[playerId];
  if (playerData) {
    return [playerId, ...playerData.allied_player_ids];
  }
  return [playerId, ...(state.deployable_owner_ids ?? [])];
}

/** サーバーから状態を取得するため、クライアント側のデフォルトは空。マップは state 受信後に全マス表示。 */
export const DEFAULT_GAME_STATE: GameState = {
  turn: 1,
  phase: "idle",
  log: [],
  territories: [],
  players: {},
  deployable_owner_ids: [],
  inventory: [],
  facilities: [],
  owned_cards: [],
};

/** レベル → 地形名（マスイラストとリンク） */
export const LEVEL_TERRAIN: Record<number, string> = {
  1: "平原",
  2: "丘陵",
  3: "森",
  4: "山地",
  5: "山岳",
  6: "川",
};

/** カードのステータス */
export interface CardStatsPayload {
  energy: number;
  speed: number;
  attack: number;
  magic: number;
  defense: number;
  magic_defense: number;
}

/** スキルデータ（サーバー送信用） */
export interface SkillDataPayload {
  passive_id?: string;
  active_id: string;
  unique_id?: string;
}

/** クライアントからサーバーへ送る行動（サーバーと同一構造） */
export type Action =
  | { action: "end_turn" }
  | {
    action: "deploy";
    territory_id: string;
    count: number;
    /** 援軍の体ごとのエナジー */
    energy_per_body?: number[];
    /** 援軍の体ごとの表示名 */
    body_names?: string[];
  }
  | {
    action: "attack";
    from_territory_id: string;
    to_territory_id: string;
    count: number;
    /** 攻撃側の体ごとのエナジー（先頭から順に敵1体目・2体目…と戦闘） */
    energy_per_body?: number[];
    /** 攻撃側の体ごとの表示名（戦闘ログ用） */
    body_names?: string[];
    /** 攻撃するユニットの表示名（ログ用。例: ユニット1） */
    unit_name?: string;
    /** 攻撃側の体ごとのSPEED */
    speed_per_body?: number[];
    /** 攻撃側の体ごとのスキルデータ */
    skills_per_body?: SkillDataPayload[];
    /** 攻撃側の体ごとの全ステータス */
    stats_per_body?: CardStatsPayload[];
  };

export const END_TURN_ACTION: Action = { action: "end_turn" };

export function deployAction(
  territoryId: string,
  count: number,
  energyPerBody?: number[],
  bodyNames?: string[]
): Action {
  return {
    action: "deploy",
    territory_id: territoryId,
    count,
    ...(energyPerBody != null && energyPerBody.length === count && { energy_per_body: energyPerBody }),
    ...(bodyNames != null && bodyNames.length === count && { body_names: bodyNames }),
  };
}

export function attackAction(
  fromTerritoryId: string,
  toTerritoryId: string,
  count: number,
  energyPerBody?: number[],
  bodyNames?: string[],
  unitName?: string,
  speedPerBody?: number[],
  skillsPerBody?: SkillDataPayload[],
  statsPerBody?: CardStatsPayload[]
): Action {
  return {
    action: "attack",
    from_territory_id: fromTerritoryId,
    to_territory_id: toTerritoryId,
    count,
    ...(energyPerBody != null && energyPerBody.length === count && { energy_per_body: energyPerBody }),
    ...(bodyNames != null && bodyNames.length === count && { body_names: bodyNames }),
    ...(unitName != null && unitName !== "" && { unit_name: unitName }),
    ...(speedPerBody != null && speedPerBody.length === count && { speed_per_body: speedPerBody }),
    ...(skillsPerBody != null && skillsPerBody.length === count && { skills_per_body: skillsPerBody }),
    ...(statsPerBody != null && statsPerBody.length === count && { stats_per_body: statsPerBody }),
  };
}
