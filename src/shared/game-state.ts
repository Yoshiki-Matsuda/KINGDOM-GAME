/**
 * ゲーム状態の共通型（サーバーと同一構造）
 * 最終形 PvPvE を想定。
 */

/** 遺跡の難易度 */
export type RuinDifficulty = "normal" | "rare" | "legendary";

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
  /** 敵の魔獣数（3体） */
  enemy_monster_counts: number[];
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
  /** 体ごとのモンスター数。戦闘はこの順で1体ずつ行う */
  body_monster_counts?: number[] | null;
  /** 体ごとの表示名（戦闘ログ用） */
  body_names?: string[] | null;
  /** 前線基地フラグ（KC準拠: 占領地に建設して前線を拡大） */
  is_base?: boolean;
  /** 遺跡情報（存在する場合） */
  ruin?: RuinInfo | null;
  /** PvP拠点・塔の耐久（0で未使用） */
  durability?: number;
  max_durability?: number;
  /** 塔レベル 1-7、通常マスは 0 */
  tower_level?: number;
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

/** KC準拠の4種基本資源 + ゴールド（フリマ用通貨） */
export interface Resources {
  food: number;
  wood: number;
  stone: number;
  iron: number;
  gold: number;
}

/** フリーマーケット出品物の種別 */
export type MarketItemType =
  | { type: "card"; card_id: number }
  | { type: "item"; item_id: string; count: number }
  | { type: "resource"; resource_type: string; amount: number };

/** フリーマーケットの出品情報 */
export interface MarketListing {
  listing_id: string;
  seller_id: string;
  item: MarketItemType;
  price: number;
  listed_at: number;
}

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
  /** 4種基本資源 */
  resources?: Resources;
  card_levels?: number[];
  card_exp?: number[];
  card_stamina?: number[];
  exploration_level?: number;
  exploration_score?: number;
  unit_cost_cap?: number;
  dungeon_points?: number;
  charge_points?: number;
  /** 進行中の探索派遣 */
  explorations?: ExplorationMission[];
}

/** 探索ミッション（サーバーと同構造） */
export interface ExplorationMission {
  mission_id: string;
  territory_id: string;
  started_at: number;
  completes_at: number;
  card_indices: number[];
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
  /** 4種基本資源（シングルプレイ用） */
  resources?: Resources;
  /** 同盟一覧 */
  alliances?: Alliance[];
  /** シーズン情報 */
  season?: SeasonInfo;
  /** フリーマーケット出品一覧 */
  market_listings?: MarketListing[];
  card_levels?: number[];
  card_exp?: number[];
  card_stamina?: number[];
  exploration_level?: number;
  exploration_score?: number;
  unit_cost_cap?: number;
  dungeon_points?: number;
  charge_points?: number;
  explorations?: ExplorationMission[];
}

/** KC準拠の同盟データ */
export interface Alliance {
  id: string;
  name: string;
  leader_id: string;
  member_ids: string[];
  territory_points: number;
  level?: number;
  donated_total?: number;
  parent_alliance_id?: string | null;
  child_alliance_ids?: string[];
}

/** KC準拠のシーズン情報 */
export interface SeasonInfo {
  season_number: number;
  started_at: number;
  duration_ms: number;
}

/** プレイヤーデータを取得（後方互換対応） */
export function getPlayerData(state: GameState, playerId: string = DEFAULT_PLAYER_ID): PlayerData | null {
  return state.players?.[playerId] ?? null;
}

/** プレイヤーのインベントリを取得（後方互換対応） */
export function getPlayerInventory(state: GameState, playerId: string = DEFAULT_PLAYER_ID): InventoryItem[] {
  return state.players?.[playerId]?.inventory ?? state.inventory ?? [];
}

/** プレイヤーの施設を取得 */
export function getPlayerFacilities(state: GameState, playerId: string = DEFAULT_PLAYER_ID): BuiltFacility[] {
  return state.players?.[playerId]?.facilities ?? state.facilities ?? [];
}

/** プレイヤーの所持カードを取得（`players` が空配列でもトップレベル `owned_cards` にフォールバック） */
export function getPlayerOwnedCards(state: GameState, playerId: string = DEFAULT_PLAYER_ID): number[] {
  const fromPlayer = state.players?.[playerId]?.owned_cards;
  const top = state.owned_cards ?? [];
  if (Array.isArray(fromPlayer) && fromPlayer.length > 0) return fromPlayer;
  if (top.length > 0) return top;
  return Array.isArray(fromPlayer) ? fromPlayer : [];
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
  monster_count: number;
  speed: number;
  attack: number;
  intelligence: number;
  defense: number;
  magic_defense: number;
  /** 射程 (1=近接, 2=中距離, 3=遠距離) */
  range?: number;
  /** ユニット編成コスト */
  cost?: number;
  /** 占拠力 */
  occupation_power?: number;
}

/** スキルデータ（サーバー送信用） */
export interface SkillDataPayload {
  passive_id?: string;
  active_id: string;
  unique_id?: string;
  skill_level?: number;
}

/** クライアントからサーバーへ送る行動（サーバーと同一構造） */
export type Action =
  | { action: "end_turn" }
  | {
    action: "deploy";
    territory_id: string;
    count: number;
    /** 援軍の体ごとのモンスター数 */
    monsters_per_body?: number[];
    /** 援軍の体ごとの表示名 */
    body_names?: string[];
  }
  | {
    action: "attack";
    from_territory_id: string;
    to_territory_id: string;
    count: number;
    /** 攻撃側の体ごとのモンスター数（先頭から順に敵1体目・2体目…と戦闘） */
    monsters_per_body?: number[];
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
    /** 所持カード配列上のインデックス（スタミナ・XP用） */
    owned_card_indices?: number[];
  }
  | {
    action: "build_base";
    territory_id: string;
  }
  | {
      action: "synthesize_card";
      base_card_index: number;
      material_card_indices: number[];
    }
  | { action: "create_alliance"; name: string }
  | { action: "join_alliance"; alliance_id: string }
  | { action: "leave_alliance" }
  | { action: "list_on_flea_market"; item: MarketItemType; price: number }
  | { action: "buy_from_flea_market"; listing_id: string }
  | { action: "cancel_flea_market_listing"; listing_id: string }
  | { action: "start_exploration"; territory_id: string; card_indices: number[] }
  | { action: "collect_exploration"; mission_id: string }
  | { action: "donate_alliance"; food: number; wood: number; stone: number; iron: number };

export const END_TURN_ACTION: Action = { action: "end_turn" };

export function buildBaseAction(territoryId: string): Action {
  return { action: "build_base", territory_id: territoryId };
}

export function deployAction(
  territoryId: string,
  count: number,
  monstersPerBody?: number[],
  bodyNames?: string[]
): Action {
  return {
    action: "deploy",
    territory_id: territoryId,
    count,
    ...(monstersPerBody != null && monstersPerBody.length === count && { monsters_per_body: monstersPerBody }),
    ...(bodyNames != null && bodyNames.length === count && { body_names: bodyNames }),
  };
}

export function attackAction(
  fromTerritoryId: string,
  toTerritoryId: string,
  count: number,
  monstersPerBody?: number[],
  bodyNames?: string[],
  unitName?: string,
  speedPerBody?: number[],
  skillsPerBody?: SkillDataPayload[],
  statsPerBody?: CardStatsPayload[],
  ownedCardIndices?: number[]
): Action {
  return {
    action: "attack",
    from_territory_id: fromTerritoryId,
    to_territory_id: toTerritoryId,
    count,
    ...(monstersPerBody != null && monstersPerBody.length === count && { monsters_per_body: monstersPerBody }),
    ...(bodyNames != null && bodyNames.length === count && { body_names: bodyNames }),
    ...(unitName != null && unitName !== "" && { unit_name: unitName }),
    ...(speedPerBody != null && speedPerBody.length === count && { speed_per_body: speedPerBody }),
    ...(skillsPerBody != null && skillsPerBody.length === count && { skills_per_body: skillsPerBody }),
    ...(statsPerBody != null && statsPerBody.length === count && { stats_per_body: statsPerBody }),
    ...(ownedCardIndices != null &&
      ownedCardIndices.length === count && { owned_card_indices: ownedCardIndices }),
  };
}
