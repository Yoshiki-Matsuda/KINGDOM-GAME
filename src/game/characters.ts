import { getCharacterSkills, type CharacterSkills, type SkillData, getCharacterSkillData } from "./skills";
import { getRarityClass } from "../shared/rarity-colors";

/** 3体で1ユニットが編成される */
export const BODIES_PER_UNIT = 3;

/** キャラ1体あたりの仮の魔獣数（必ず勝てる数値） */
export const DEFAULT_BODY_MONSTER_COUNT = 10;

/** キャラ1体あたりのデフォルト速さ（未設定時のフォールバック） */
export const DEFAULT_BODY_SPEED = 5;

/** 移動時間の基本係数（秒/マス）。速さで割る */
export const BASE_TRAVEL_TIME_PER_TILE = 2.0;

/** 魔獣の基本ステータス定義 */
export interface CardStats {
  /** 魔獣数（HP/ベース係数） */
  monster_count: number;
  /** 速さ（行動順・移動時間） */
  speed: number;
  /** 攻撃力（物理ダメージ） */
  attack: number;
  /** 知力（スキル効果に影響） */
  intelligence: number;
  /** 防御力（物理防御） */
  defense: number;
  /** 魔法防御力 */
  magicDefense: number;
  /** 射程 (1=近接, 2=中距離, 3=遠距離) */
  range: number;
  /** ユニット編成コスト (KC) */
  cost: number;
  /** 占拠力 */
  occupationPower: number;
}

export const DEFAULT_CARD_STATS: CardStats = {
  monster_count: 10,
  speed: 5,
  attack: 5,
  intelligence: 5,
  defense: 3,
  magicDefense: 3,
  range: 1,
  cost: 1.5,
  occupationPower: 100,
};

/** 魔獣のレアリティ */
export type CardRarity = "common" | "uncommon" | "rare" | "epic" | "legendary";

/** KC準拠の7種族 */
export type Race = "beast" | "demihuman" | "demon" | "dragon" | "giant" | "spirit" | "undead";

/** キャラクターデータ */
export interface CharacterData {
  index: number;
  name: string;
  race: Race;
  stats: CardStats;
  skills: CharacterSkills;
}

/** 統合キャラクター定義: 全属性を1レコードに集約 */
interface CharacterDef {
  name: string;
  race: Race;
  rarity: CardRarity;
  stats: Partial<CardStats>;
}

const CHARACTERS: Record<number, CharacterDef> = {
  // === プレイヤー初期魔獣（KC 7種族） ===
  0: { name: "ダイアウルフ", race: "beast", rarity: "rare", stats: { monster_count: 15, attack: 9, intelligence: 3, defense: 4, magicDefense: 3, speed: 6 } },
  1: { name: "ゴブリンウォリアー", race: "demihuman", rarity: "rare", stats: { monster_count: 18, attack: 7, intelligence: 4, defense: 6, magicDefense: 3, speed: 5 } },
  2: { name: "インプ", race: "demon", rarity: "rare", stats: { monster_count: 10, attack: 4, intelligence: 10, defense: 2, magicDefense: 6, speed: 7, range: 2 } },
  3: { name: "ワイバーン", race: "dragon", rarity: "rare", stats: { monster_count: 12, attack: 8, intelligence: 6, defense: 5, magicDefense: 5, speed: 5, range: 2 } },
  4: { name: "ゴーレム", race: "giant", rarity: "rare", stats: { monster_count: 16, attack: 6, intelligence: 2, defense: 10, magicDefense: 4, speed: 3 } },
  5: { name: "サラマンダー", race: "spirit", rarity: "rare", stats: { monster_count: 11, attack: 5, intelligence: 9, defense: 3, magicDefense: 7, speed: 5, range: 2 } },
  6: { name: "スケルトンソルジャー", race: "undead", rarity: "rare", stats: { monster_count: 14, attack: 7, intelligence: 3, defense: 7, magicDefense: 4, speed: 4 } },
  7: { name: "ヘルハウンド", race: "beast", rarity: "rare", stats: { monster_count: 13, attack: 8, intelligence: 4, defense: 3, magicDefense: 3, speed: 7 } },
  8: { name: "リザードマン", race: "demihuman", rarity: "rare", stats: { monster_count: 14, attack: 8, intelligence: 5, defense: 5, magicDefense: 4, speed: 5 } },
  9: { name: "トレント", race: "spirit", rarity: "rare", stats: { monster_count: 16, attack: 5, intelligence: 7, defense: 8, magicDefense: 6, speed: 3 } },
  // === フィールド敵（Lv1〜6） ===
  10: { name: "ゴブリン", race: "demihuman", rarity: "common", stats: { monster_count: 2, attack: 3, intelligence: 2, defense: 2, magicDefense: 1, speed: 3 } },
  11: { name: "コボルド", race: "demihuman", rarity: "common", stats: { monster_count: 4, attack: 5, intelligence: 3, defense: 3, magicDefense: 2, speed: 4 } },
  12: { name: "オーク", race: "demihuman", rarity: "uncommon", stats: { monster_count: 6, attack: 8, intelligence: 4, defense: 6, magicDefense: 3, speed: 3 } },
  13: { name: "スケルトン", race: "undead", rarity: "uncommon", stats: { monster_count: 8, attack: 10, intelligence: 6, defense: 8, magicDefense: 5, speed: 4 } },
  14: { name: "トロール", race: "giant", rarity: "rare", stats: { monster_count: 12, attack: 15, intelligence: 3, defense: 12, magicDefense: 4, speed: 3 } },
  15: { name: "ドレイク", race: "dragon", rarity: "epic", stats: { monster_count: 15, attack: 20, intelligence: 15, defense: 15, magicDefense: 10, speed: 6, range: 2 } },
  // === 遺跡敵（ノーマル） ===
  20: { name: "ストーンゴーレム", race: "giant", rarity: "uncommon", stats: { monster_count: 8, attack: 6, intelligence: 2, defense: 10, magicDefense: 4, speed: 2 } },
  21: { name: "ゴースト", race: "undead", rarity: "uncommon", stats: { monster_count: 6, attack: 4, intelligence: 8, defense: 3, magicDefense: 8, speed: 5, range: 2 } },
  22: { name: "スケルトンナイト", race: "undead", rarity: "uncommon", stats: { monster_count: 10, attack: 10, intelligence: 3, defense: 8, magicDefense: 4, speed: 4 } },
  23: { name: "コカトリス", race: "beast", rarity: "rare", stats: { monster_count: 12, attack: 7, intelligence: 8, defense: 4, magicDefense: 6, speed: 5 } },
  24: { name: "ミミック", race: "demon", rarity: "rare", stats: { monster_count: 7, attack: 8, intelligence: 5, defense: 5, magicDefense: 5, speed: 7 } },
  25: { name: "ポイズンスパイダー", race: "beast", rarity: "uncommon", stats: { monster_count: 5, attack: 6, intelligence: 4, defense: 3, magicDefense: 3, speed: 6 } },
  // === 遺跡敵（レア） ===
  30: { name: "ダークウィザード", race: "demon", rarity: "rare", stats: { monster_count: 12, attack: 12, intelligence: 10, defense: 6, magicDefense: 8, speed: 5, range: 2 } },
  31: { name: "ガーゴイル", race: "demon", rarity: "rare", stats: { monster_count: 14, attack: 14, intelligence: 4, defense: 12, magicDefense: 6, speed: 4 } },
  32: { name: "シャドウアサシン", race: "demon", rarity: "rare", stats: { monster_count: 10, attack: 16, intelligence: 2, defense: 6, magicDefense: 4, speed: 8 } },
  33: { name: "フレイムスピリット", race: "spirit", rarity: "rare", stats: { monster_count: 9, attack: 6, intelligence: 14, defense: 4, magicDefense: 10, speed: 5, range: 2 } },
  34: { name: "アイスエレメンタル", race: "spirit", rarity: "rare", stats: { monster_count: 9, attack: 6, intelligence: 14, defense: 4, magicDefense: 10, speed: 5, range: 2 } },
  35: { name: "デスナイト", race: "undead", rarity: "epic", stats: { monster_count: 16, attack: 12, intelligence: 6, defense: 14, magicDefense: 8, speed: 4 } },
  36: { name: "ヒュドラ", race: "dragon", rarity: "epic", stats: { monster_count: 14, attack: 14, intelligence: 12, defense: 10, magicDefense: 10, speed: 4 } },
  37: { name: "ミノタウロス", race: "giant", rarity: "epic", stats: { monster_count: 14, attack: 16, intelligence: 4, defense: 12, magicDefense: 6, speed: 5 } },
  // === 遺跡敵（レジェンダリー） ===
  40: { name: "ニーズヘッグ", race: "dragon", rarity: "legendary", stats: { monster_count: 30, attack: 25, intelligence: 18, defense: 20, magicDefense: 15, speed: 4 } },
  41: { name: "ヴァンパイアロード", race: "undead", rarity: "legendary", stats: { monster_count: 22, attack: 18, intelligence: 20, defense: 14, magicDefense: 18, speed: 6 } },
  42: { name: "リッチ", race: "undead", rarity: "legendary", stats: { monster_count: 22, attack: 15, intelligence: 25, defense: 12, magicDefense: 20, speed: 5, range: 2 } },
  43: { name: "タイタン", race: "giant", rarity: "legendary", stats: { monster_count: 35, attack: 22, intelligence: 12, defense: 25, magicDefense: 18, speed: 2 } },
  // === 収集可能魔獣: 獣族 ===
  50: { name: "バット", race: "beast", rarity: "common", stats: { monster_count: 8, attack: 4, intelligence: 2, defense: 3, magicDefense: 2, speed: 8 } },
  51: { name: "ジャイアントバット", race: "beast", rarity: "uncommon", stats: { monster_count: 10, attack: 6, intelligence: 3, defense: 5, magicDefense: 3, speed: 7 } },
  52: { name: "ヴァンパイアバット", race: "beast", rarity: "uncommon", stats: { monster_count: 12, attack: 7, intelligence: 4, defense: 5, magicDefense: 4, speed: 7 } },
  53: { name: "カマソッツ", race: "beast", rarity: "rare", stats: { monster_count: 14, attack: 9, intelligence: 5, defense: 6, magicDefense: 5, speed: 8, range: 2 } },
  54: { name: "ボーゲスト", race: "beast", rarity: "uncommon", stats: { monster_count: 11, attack: 7, intelligence: 4, defense: 6, magicDefense: 3, speed: 5 } },
  55: { name: "ガイトラッシュ", race: "beast", rarity: "rare", stats: { monster_count: 14, attack: 10, intelligence: 5, defense: 7, magicDefense: 4, speed: 6 } },
  56: { name: "レウクロコタ", race: "beast", rarity: "epic", stats: { monster_count: 18, attack: 14, intelligence: 6, defense: 8, magicDefense: 5, speed: 9 } },
  // === 収集可能魔獣: 亜人族 ===
  60: { name: "ゴブリンアーチャー", race: "demihuman", rarity: "common", stats: { monster_count: 6, attack: 5, intelligence: 2, defense: 2, magicDefense: 1, speed: 4, range: 3, cost: 1.0 } },
  61: { name: "ゴブリンコック", race: "demihuman", rarity: "common", stats: { monster_count: 10, attack: 5, intelligence: 5, defense: 5, magicDefense: 4, speed: 4, range: 2 } },
  62: { name: "ホブゴブリン", race: "demihuman", rarity: "uncommon", stats: { monster_count: 12, attack: 7, intelligence: 5, defense: 8, magicDefense: 5, speed: 5, range: 2 } },
  63: { name: "オークアーマーナイト", race: "demihuman", rarity: "uncommon", stats: { monster_count: 14, attack: 8, intelligence: 4, defense: 10, magicDefense: 5, speed: 3 } },
  64: { name: "ゴブリンソードマン", race: "demihuman", rarity: "uncommon", stats: { monster_count: 12, attack: 9, intelligence: 5, defense: 6, magicDefense: 4, speed: 6 } },
  65: { name: "ホブゴブリンダークナイト", race: "demihuman", rarity: "rare", stats: { monster_count: 16, attack: 12, intelligence: 4, defense: 11, magicDefense: 6, speed: 5, range: 2 } },
  66: { name: "ゴブリンプリンセス", race: "demihuman", rarity: "epic", stats: { monster_count: 15, attack: 12, intelligence: 10, defense: 10, magicDefense: 8, speed: 5 } },
  // === 収集可能魔獣: 魔族 ===
  70: { name: "レッサーデーモン", race: "demon", rarity: "common", stats: { monster_count: 8, attack: 6, intelligence: 5, defense: 4, magicDefense: 5, speed: 5 } },
  71: { name: "サキュバス", race: "demon", rarity: "uncommon", stats: { monster_count: 10, attack: 5, intelligence: 10, defense: 4, magicDefense: 8, speed: 6, range: 2 } },
  72: { name: "ナイトメア", race: "demon", rarity: "uncommon", stats: { monster_count: 12, attack: 8, intelligence: 6, defense: 6, magicDefense: 5, speed: 7 } },
  73: { name: "アークデーモン", race: "demon", rarity: "rare", stats: { monster_count: 16, attack: 13, intelligence: 8, defense: 10, magicDefense: 8, speed: 5 } },
  74: { name: "リリス", race: "demon", rarity: "epic", stats: { monster_count: 14, attack: 10, intelligence: 16, defense: 8, magicDefense: 14, speed: 7, range: 2 } },
  // === 収集可能魔獣: 竜族 ===
  80: { name: "リンドヴルム", race: "dragon", rarity: "common", stats: { monster_count: 8, attack: 7, intelligence: 4, defense: 5, magicDefense: 4, speed: 4 } },
  81: { name: "シーサーペント", race: "dragon", rarity: "uncommon", stats: { monster_count: 12, attack: 8, intelligence: 6, defense: 7, magicDefense: 6, speed: 5 } },
  82: { name: "ファイアドレイク", race: "dragon", rarity: "rare", stats: { monster_count: 14, attack: 11, intelligence: 9, defense: 8, magicDefense: 7, speed: 5, range: 2 } },
  83: { name: "バハムート", race: "dragon", rarity: "epic", stats: { monster_count: 20, attack: 18, intelligence: 14, defense: 16, magicDefense: 12, speed: 4, range: 2 } },
  // === 収集可能魔獣: 巨人族 ===
  90: { name: "オーガ", race: "giant", rarity: "common", stats: { monster_count: 10, attack: 7, intelligence: 2, defense: 6, magicDefense: 2, speed: 3 } },
  91: { name: "サイクロプス", race: "giant", rarity: "uncommon", stats: { monster_count: 14, attack: 9, intelligence: 3, defense: 9, magicDefense: 4, speed: 3 } },
  92: { name: "アイアンゴーレム", race: "giant", rarity: "rare", stats: { monster_count: 18, attack: 10, intelligence: 3, defense: 14, magicDefense: 6, speed: 2 } },
  93: { name: "ギガース", race: "giant", rarity: "epic", stats: { monster_count: 22, attack: 16, intelligence: 5, defense: 18, magicDefense: 10, speed: 3 } },
  // === 収集可能魔獣: 精霊族 ===
  100: { name: "ウィスプ", race: "spirit", rarity: "common", stats: { monster_count: 6, attack: 3, intelligence: 7, defense: 2, magicDefense: 6, speed: 6, range: 2 } },
  101: { name: "シルフ", race: "spirit", rarity: "uncommon", stats: { monster_count: 8, attack: 4, intelligence: 9, defense: 3, magicDefense: 7, speed: 7, range: 2 } },
  102: { name: "ウンディーネ", race: "spirit", rarity: "uncommon", stats: { monster_count: 10, attack: 5, intelligence: 10, defense: 5, magicDefense: 9, speed: 5, range: 2 } },
  103: { name: "イフリート", race: "spirit", rarity: "rare", stats: { monster_count: 14, attack: 12, intelligence: 11, defense: 7, magicDefense: 8, speed: 5, range: 2 } },
  104: { name: "フェニックス", race: "spirit", rarity: "epic", stats: { monster_count: 16, attack: 10, intelligence: 15, defense: 8, magicDefense: 14, speed: 6, range: 2 } },
  // === 収集可能魔獣: 不死族 ===
  110: { name: "ゾンビ", race: "undead", rarity: "common", stats: { monster_count: 10, attack: 5, intelligence: 1, defense: 6, magicDefense: 2, speed: 2 } },
  111: { name: "レイス", race: "undead", rarity: "uncommon", stats: { monster_count: 8, attack: 4, intelligence: 8, defense: 3, magicDefense: 9, speed: 5, range: 2 } },
  112: { name: "ワイト", race: "undead", rarity: "uncommon", stats: { monster_count: 12, attack: 8, intelligence: 5, defense: 7, magicDefense: 5, speed: 4 } },
  113: { name: "ドゥラハン", race: "undead", rarity: "rare", stats: { monster_count: 14, attack: 11, intelligence: 4, defense: 10, magicDefense: 6, speed: 5 } },
  114: { name: "エルダーリッチ", race: "undead", rarity: "epic", stats: { monster_count: 16, attack: 8, intelligence: 18, defense: 8, magicDefense: 16, speed: 4, range: 2, cost: 1.0 } },
};

/** キャラクターの完全ステータスを取得 */
export function getCharacterStats(index: number): CardStats {
  const custom = CHARACTERS[index]?.stats ?? {};
  return { ...DEFAULT_CARD_STATS, ...custom };
}

/** キャラクター名を取得（統合レコードから） */
export function getBodyDisplayName(index: number): string {
  return CHARACTERS[index]?.name ?? `キャラ${index + 1}`;
}

/** `public/cards/character-{id}.png` が存在する魔獣ID */
export const ILLUSTRATED_CARD_IDS: readonly number[] = [
  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
  50, 51, 52, 53, 54, 55, 56,
  60, 61, 62, 63, 64, 65, 66,
  70, 71, 72, 73, 74,
  80, 81, 82, 83,
  90, 91, 92, 93,
  100, 101, 102, 103, 104,
  110, 111, 112, 113, 114,
];

const ILLUSTRATED_CARD_ID_SET = new Set(ILLUSTRATED_CARD_IDS);

/** 魔獣にイラスト画像が紐づいているか */
export function hasCharacterIllustration(index: number): boolean {
  return ILLUSTRATED_CARD_ID_SET.has(index);
}

/** 魔獣イラストのパス（キャラクターインデックス→画像URL） */
export function getCharacterIllustrationPath(index: number): string {
  if (!hasCharacterIllustration(index)) {
    return "/cards/placeholder.svg";
  }
  return `/cards/character-${index}.png`;
}

/** 魔獣のレアリティを取得（統合レコードから） */
export function getCardRarity(index: number): CardRarity {
  return CHARACTERS[index]?.rarity ?? "common";
}

export { getRarityColor, getRarityClass } from "../shared/rarity-colors";
export function getCardRarityClass(cardId: number): string {
  return getRarityClass(getCardRarity(cardId));
}

/** 同一 card_id の所持スロット数 */
export function countOwnedSlotsByCardId(owned: number[], cardId: number): number {
  return owned.filter((id) => id === cardId).length;
}

/** card_id の代表スロット（最初の出現インデックス） */
export function getCanonicalSlotForCardId(owned: number[], cardId: number): number | undefined {
  const idx = owned.indexOf(cardId);
  return idx >= 0 ? idx : undefined;
}

/** 魔獣一覧用: イラストあり種族ごとに代表スロットを返す */
export function getUniqueIllustratedSpeciesSlots(owned: number[]): number[] {
  const seen = new Set<number>();
  const slots: number[] = [];
  for (let i = 0; i < owned.length; i++) {
    const cardId = owned[i];
    if (!hasCharacterIllustration(cardId) || seen.has(cardId)) continue;
    seen.add(cardId);
    slots.push(i);
  }
  return slots;
}

/** 魔獣の種族を取得（統合レコードから） */
export function getCardRace(index: number): Race {
  return CHARACTERS[index]?.race ?? "demihuman";
}

/** 魔獣名からインデックスを取得 */
export function getCardIndexByName(name: string): number | undefined {
  for (const [idx, def] of Object.entries(CHARACTERS)) {
    if (def.name === name) return parseInt(idx);
  }
  return undefined;
}

/** キャラクターの完全データを取得 */
export function getCharacterData(index: number): CharacterData {
  return {
    index,
    name: getBodyDisplayName(index),
    race: getCardRace(index),
    stats: getCharacterStats(index),
    skills: getCharacterSkills(index),
  };
}

/** 後方互換: monster_count, speedを指定してデータ取得（既存コードとの互換用） */
export function getCharacterDataLegacy(index: number, monsterCount: number, speed: number): CharacterData {
  const stats = getCharacterStats(index);
  return {
    index,
    name: getBodyDisplayName(index),
    race: getCardRace(index),
    stats: { ...stats, monster_count: monsterCount, speed },
    skills: getCharacterSkills(index),
  };
}

/** キャラクターのスキルデータを取得（サーバー送信用） */
export { getCharacterSkillData };
export type { SkillData };
