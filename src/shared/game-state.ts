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

/** レベル 1=平原 〜 9=深域。地形見た目・PvE 敵の強さと連動 */
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
export interface FacilityPosition {
  col: number;
  row: number;
}

export interface BuiltFacility {
  facility_id: string;
  level: number;
  /** 建設完了時刻（Unix timestamp ms）。nullなら完了済み */
  build_complete_at?: number | null;
  /** 配置座標（ホームマップ上） */
  position?: FacilityPosition;
}

/** デフォルトの操作プレイヤーID */
export const DEFAULT_PLAYER_ID = "player";

/** 所持魔獣1枠あたりの魔獣数の共通下限（サーバー `MIN_MONSTER_COUNT_PER_CARD_SLOT` と一致） */
export const MIN_MONSTER_COUNT_PER_CARD_SLOT = 1;

/** 所持魔獣1枠あたりの魔獣数の共通上限（サーバー `MAX_MONSTER_COUNT_PER_CARD_SLOT` と一致） */
export const MAX_MONSTER_COUNT_PER_CARD_SLOT = 9999;

/** 魔獣1体生産あたりの食料消費（サーバー `FOOD_PER_MONSTER_PRODUCE` と一致） */
export const FOOD_PER_MONSTER_PRODUCE = 2;

/** KC準拠の4種基本資源 + ゴールド（フリマ用通貨） */
export interface Resources {
  food: number;
  wood: number;
  stone: number;
  iron: number;
  gold: number;
}

/** フリーマーケットで売買できる基本資源 */
export type BasicResourceType = "food" | "wood" | "stone" | "iron";

/** フリーマーケット出品物の種別 */
export type MarketItemType =
  | { type: "card"; card_id: number }
  | { type: "item"; item_id: string; count: number }
  | { type: "resource"; resource_type: BasicResourceType; amount: number };

/** フリーマーケットの出品情報 */
export interface MarketListing {
  listing_id: string;
  seller_id: string;
  item: MarketItemType;
  price: number;
  listed_at: number;
}

/** 保存用ユニット編成 */
export interface StoredFormedUnit {
  id: string;
  name: string;
  indices: [number, number, number];
}

/** レベルアップ等で振り分けたステータスボーナス（所持スロットごと） */
export interface CardStatBonuses {
  speed: number;
  attack: number;
  intelligence: number;
  defense: number;
  magic_defense: number;
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
  /** 所持魔獣（データ列 `owned_cards`・各要素は魔獣マスタID） */
  owned_cards: number[];
  /** 援軍を送れる他プレイヤーのID（クラン・配下など） */
  allied_player_ids: string[];
  /** 4種基本資源 */
  resources: Resources;
  card_levels?: number[];
  card_exp?: number[];
  card_stamina?: number[];
  /** 所持魔獣スロットごとの未配分ステータスポイント（Lvアップで+10） */
  card_status_points?: number[];
  /** 所持魔獣スロットごとの配分済みステータスボーナス */
  card_stat_bonuses?: CardStatBonuses[];
  /** 所持魔獣スロットごとの休息解除時刻（Unix timestamp ms）。未来なら休息中 */
  card_rest_until?: number[];
  /** 所持魔獣スロットごとの覚醒フラグ（KC準拠: Lv99超え可） */
  card_awakened?: boolean[];
  /** 所持魔獣スロットごとの強化魔獣(★)フラグ */
  card_enhanced?: boolean[];
  /** 所持魔獣スロットごとの現在魔獣数（本拠の body_monster_counts と対応） */
  card_monster_counts?: number[];
  exploration_level?: number;
  exploration_score?: number;
  unit_cost_cap?: number;
  dungeon_points?: number;
  charge_points?: number;
  /** 進行中の遠征（攻撃・援軍・探索・帰還） */
  marches?: MarchMission[];
  /** ユニット編成（永続化） */
  formed_units?: StoredFormedUnit[];
}

export type MarchKind = "attack" | "deploy" | "explore" | "return";

/** 遠征ミッション（サーバーと同構造） */
export interface MarchMission {
  march_id: string;
  kind: MarchKind;
  from_territory_id: string;
  to_territory_id: string;
  started_at: number;
  arrives_at: number;
  count: number;
  monsters_per_body?: number[];
  body_names?: string[];
  unit_name?: string;
  speed_per_body?: number[];
  owned_card_indices?: number[];
  formed_unit_id?: string;
}

/** 魔獣スタミナ上限（サーバー `MAX_CARD_STAMINA` と一致） */
export const MAX_CARD_STAMINA = 100;

/** KC準拠: 次レベルに必要な経験値（サーバー `exp_needed_for_level` と同式） */
export function expNeededForLevel(currentLevel: number): number {
  const lv = Math.max(1, currentLevel);
  return Math.round(100 * Math.pow(lv, 1.4));
}

export function getPlayerCardLevel(state: GameState, bodySlot: number, playerId: string = DEFAULT_PLAYER_ID): number {
  return state.players[playerId]?.card_levels?.[bodySlot] ?? 1;
}

export function getPlayerCardExp(state: GameState, bodySlot: number, playerId: string = DEFAULT_PLAYER_ID): number {
  return state.players[playerId]?.card_exp?.[bodySlot] ?? 0;
}

export function getPlayerCardStamina(state: GameState, bodySlot: number, playerId: string = DEFAULT_PLAYER_ID): number {
  return state.players[playerId]?.card_stamina?.[bodySlot] ?? MAX_CARD_STAMINA;
}

export function getPlayerMarches(state: GameState, playerId: string = DEFAULT_PLAYER_ID): MarchMission[] {
  return state.players[playerId]?.marches ?? [];
}

/** KC準拠: 探索の同時派遣数（サーバー `exploration_max_slots` と一致） */
export function explorationMaxSlots(explorationLevel: number): number {
  const lv = Math.max(0, explorationLevel);
  if (lv <= 19) return 1;
  if (lv <= 39) return 2;
  if (lv <= 59) return 3;
  if (lv <= 79) return 4;
  if (lv <= 99) return 5;
  return 6;
}

/** 未到着の探索遠征で派遣中の体数 */
export function activeExploreBodiesInFlight(marches: MarchMission[], now: number = Date.now()): number {
  return marches
    .filter((m) => m.kind === "explore" && m.arrives_at > now)
    .reduce((sum, m) => sum + (m.owned_card_indices?.length ?? m.count ?? 0), 0);
}

/** 進行中遠征（未到着）で使用中の魔獣スロット（帰還中含む） */
export function getMarchLockedCardSlots(
  state: GameState,
  playerId: string = DEFAULT_PLAYER_ID,
  now: number = Date.now(),
): Set<number> {
  const locked = new Set<number>();
  for (const march of getPlayerMarches(state, playerId)) {
    if (march.arrives_at <= now) continue;
    for (const i of march.owned_card_indices ?? []) {
      locked.add(i);
    }
  }
  return locked;
}

export function isCardSlotOnMarch(
  state: GameState,
  bodySlot: number,
  playerId: string = DEFAULT_PLAYER_ID,
  now: number = Date.now(),
): boolean {
  return getMarchLockedCardSlots(state, playerId, now).has(bodySlot);
}

/** マップ上に表示する進行中の遠征（全プレイヤー・AI） */
export interface VisibleMarch {
  march_id: string;
  owner_id: string;
  kind: MarchKind;
  home_territory_id: string;
  /** 遠征の出発領地（隣接する自領・移動時間計算用） */
  from_territory_id?: string;
  to_territory_id: string;
  arrives_at: number;
  unit_name?: string;
}

function marchToVisibleMarch(
  ownerId: string,
  player: { home_territory_id: string },
  march: MarchMission,
): VisibleMarch {
  return {
    march_id: march.march_id,
    owner_id: ownerId,
    kind: march.kind,
    home_territory_id: player.home_territory_id,
    from_territory_id: march.from_territory_id,
    to_territory_id: march.to_territory_id,
    arrives_at: march.arrives_at,
    unit_name: march.unit_name,
  };
}

/** マップ表示用の進行中遠征一覧（帰還・到着済みを除く） */
export function getMapVisibleMarches(state: GameState, now: number = Date.now()): VisibleMarch[] {
  const byId = new Map<string, VisibleMarch>();

  for (const [ownerId, player] of Object.entries(state.players)) {
    for (const march of player.marches ?? []) {
      if (march.kind === "return" || march.arrives_at <= now) continue;
      byId.set(march.march_id, marchToVisibleMarch(ownerId, player, march));
    }
  }

  for (const march of state.visible_marches ?? []) {
    if (march.kind === "return" || march.arrives_at <= now) continue;
    if (!byId.has(march.march_id)) {
      byId.set(march.march_id, march);
    }
  }

  return [...byId.values()];
}

export type GamePhase = "idle" | "player_turn" | "enemy_turn";

export interface WorldConfig {
  cols: number;
  rows: number;
  home_col: number;
  home_row: number;
  /** 地形生成シード（0 は未記録） */
  terrain_seed?: number;
}

export type AiPersonality = "aggressive" | "balanced" | "defensive";

export interface AiFaction {
  faction_id: string;
  name: string;
  personality: AiPersonality;
  home_territory_id: string;
  color: number;
}

/** 構造化ゲームイベント */
export interface GameEvent {
  id: number;
  timestamp: number;
  actor_id?: string | null;
  event_type: string;
  data: Record<string, unknown>;
  message: string;
}

export interface GameState {
  world?: WorldConfig;
  world_owner_id?: string | null;
  ai_factions?: AiFaction[];
  territories: Territory[];
  /** バックエンドで発生した行動ログ。ユーザーは閲覧のみ */
  log: GameEvent[];
  /** 全プレイヤーのデータ（プレイヤーID -> PlayerData） */
  players: Record<string, PlayerData>;
  /** 同盟一覧 */
  alliances?: Alliance[];
  /** シーズン情報 */
  season?: SeasonInfo;
  /** フリーマーケット出品一覧 */
  market_listings?: MarketListing[];
  /** マップ表示用の進行中遠征（全プレイヤー・AI） */
  visible_marches?: VisibleMarch[];
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

/** プレイヤーデータを取得 */
export function getPlayerData(state: GameState, playerId: string = DEFAULT_PLAYER_ID): PlayerData | null {
  return state.players[playerId] ?? null;
}

/** プレイヤーのインベントリを取得 */
export function getPlayerInventory(state: GameState, playerId: string = DEFAULT_PLAYER_ID): InventoryItem[] {
  return state.players[playerId]?.inventory ?? [];
}

/** プレイヤーの施設を取得 */
export function getPlayerFacilities(state: GameState, playerId: string = DEFAULT_PLAYER_ID): BuiltFacility[] {
  return state.players[playerId]?.facilities ?? [];
}

/** プレイヤーの資源を取得 */
export function getPlayerResources(state: GameState, playerId: string = DEFAULT_PLAYER_ID): Resources {
  return state.players[playerId]?.resources ?? DEFAULT_RESOURCES;
}

/** プレイヤーの食料 */
export function getPlayerFood(state: GameState, playerId: string = DEFAULT_PLAYER_ID): number {
  const fromPlayer = state.players[playerId]?.resources.food;
  if (fromPlayer != null && Number.isFinite(fromPlayer)) return fromPlayer;
  return 0;
}

/** プレイヤーの所持魔獣を取得 */
export function getPlayerOwnedCards(state: GameState, playerId: string = DEFAULT_PLAYER_ID): number[] {
  return state.players[playerId]?.owned_cards ?? [];
}

/** 所持魔獣スロットごとの魔獣数 */
export function getPlayerCardMonsterCounts(state: GameState, playerId: string = DEFAULT_PLAYER_ID): number[] {
  return state.players[playerId]?.card_monster_counts ?? [];
}

/** プレイヤーが援軍を送れるowner_idリストを取得 */
export function getDeployableOwnerIds(state: GameState, playerId: string = DEFAULT_PLAYER_ID): string[] {
  const playerData = state.players[playerId];
  if (playerData) {
    return [playerId, ...playerData.allied_player_ids];
  }
  return [playerId];
}

/** サーバーから状態を取得するため、クライアント側のデフォルトは空。 */
export const DEFAULT_RESOURCES: Resources = { food: 0, wood: 0, stone: 0, iron: 0, gold: 0 };

export const DEFAULT_WORLD_CONFIG: WorldConfig = {
  cols: 48,
  rows: 48,
  home_col: 24,
  home_row: 24,
  terrain_seed: 0,
};

export function getWorldConfig(state: GameState): WorldConfig {
  return state.world ?? DEFAULT_WORLD_CONFIG;
}

export function isAiOwnerId(ownerId: string | null | undefined): boolean {
  return !!ownerId && ownerId.startsWith("ai_");
}

export function aiFactionName(state: GameState, ownerId: string): string | null {
  if (!isAiOwnerId(ownerId)) return null;
  const suffix = ownerId.replace(/^ai_/, "");
  const faction = state.ai_factions?.find((f) => f.faction_id === suffix);
  return faction?.name ?? ownerId;
}

export const DEFAULT_GAME_STATE: GameState = {
  world: DEFAULT_WORLD_CONFIG,
  log: [],
  territories: [],
  players: {
    [DEFAULT_PLAYER_ID]: {
      player_id: DEFAULT_PLAYER_ID,
      home_territory_id: "c_24_24",
      inventory: [],
      facilities: [],
      owned_cards: [],
      allied_player_ids: [],
      resources: DEFAULT_RESOURCES,
    },
  },
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

/** 魔獣（戦闘送信用ステータス） */
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
    /** 攻撃側の体ごとの速さ。未指定時は各5として扱う */
    speed_per_body?: number[];
    /** 攻撃側の体ごとのスキルデータ */
    skills_per_body?: SkillDataPayload[];
    /** 攻撃側の体ごとの全ステータス */
    stats_per_body?: CardStatsPayload[];
    /** 所持魔獣スロットのインデックス（スタミナ・XP用） */
    owned_card_indices?: number[];
  }
  | {
    action: "build_base";
    territory_id: string;
  }
  | {
    action: "build_facility";
    facility_id: string;
    level: number;
    /** 配置座標（ホームマップ上）。サーバーは建設時間を施設定義から計算する */
    position: FacilityPosition;
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
  | {
    action: "start_march";
    kind: MarchKind;
    from_territory_id: string;
    to_territory_id: string;
    count: number;
    monsters_per_body?: number[];
    body_names?: string[];
    unit_name?: string;
    speed_per_body?: number[];
    skills_per_body?: SkillDataPayload[];
    stats_per_body?: CardStatsPayload[];
    owned_card_indices?: number[];
    formed_unit_id?: string;
  }
  | { action: "donate_alliance"; food: number; wood: number; stone: number; iron: number }
  | { action: "produce_monsters"; card_index: number; amount: number }
  | { action: "set_formed_units"; units: StoredFormedUnit[] }
  | {
    action: "allocate_card_stats";
    card_index: number;
    speed: number;
    attack: number;
    intelligence: number;
    defense: number;
    magic_defense: number;
  };

export function buildBaseAction(territoryId: string): Action {
  return { action: "build_base", territory_id: territoryId };
}

export function buildFacilityAction(
  facilityId: string,
  level: number,
  position: FacilityPosition,
): Action {
  return {
    action: "build_facility",
    facility_id: facilityId,
    level,
    position,
  };
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

export function startMarchAction(
  kind: MarchKind,
  fromTerritoryId: string,
  toTerritoryId: string,
  count: number,
  options?: {
    monstersPerBody?: number[];
    bodyNames?: string[];
    unitName?: string;
    speedPerBody?: number[];
    skillsPerBody?: SkillDataPayload[];
    statsPerBody?: CardStatsPayload[];
    ownedCardIndices?: number[];
    formedUnitId?: string;
  },
): Action {
  return {
    action: "start_march",
    kind,
    from_territory_id: fromTerritoryId,
    to_territory_id: toTerritoryId,
    count,
    ...(options?.monstersPerBody != null && options.monstersPerBody.length === count && {
      monsters_per_body: options.monstersPerBody,
    }),
    ...(options?.bodyNames != null && options.bodyNames.length === count && {
      body_names: options.bodyNames,
    }),
    ...(options?.unitName != null && options.unitName !== "" && { unit_name: options.unitName }),
    ...(options?.speedPerBody != null && options.speedPerBody.length === count && {
      speed_per_body: options.speedPerBody,
    }),
    ...(options?.skillsPerBody != null && options.skillsPerBody.length === count && {
      skills_per_body: options.skillsPerBody,
    }),
    ...(options?.statsPerBody != null && options.statsPerBody.length === count && {
      stats_per_body: options.statsPerBody,
    }),
    ...(options?.ownedCardIndices != null &&
      options.ownedCardIndices.length === count && { owned_card_indices: options.ownedCardIndices }),
    ...(options?.formedUnitId != null && { formed_unit_id: options.formedUnitId }),
  };
}

export function produceMonstersAction(cardIndex: number, amount: number): Action {
  return { action: "produce_monsters", card_index: cardIndex, amount };
}

export function setFormedUnitsAction(units: StoredFormedUnit[]): Action {
  return { action: "set_formed_units", units };
}

export function allocateCardStatsAction(
  cardIndex: number,
  delta: CardStatBonuses,
): Action {
  return {
    action: "allocate_card_stats",
    card_index: cardIndex,
    speed: delta.speed,
    attack: delta.attack,
    intelligence: delta.intelligence,
    defense: delta.defense,
    magic_defense: delta.magic_defense,
  };
}
