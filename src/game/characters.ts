import { getCharacterSkills, type CharacterSkills, type SkillData, getCharacterSkillData } from "./skills";

/** 3体で1ユニットが編成される */
export const BODIES_PER_UNIT = 3;

/** キャラ1体あたりの仮の魔獣数（必ず勝てる数値） */
export const DEFAULT_BODY_MONSTER_COUNT = 10;

/** キャラ1体あたりのデフォルトSPEED（1〜10、高いほど速い） */
export const DEFAULT_BODY_SPEED = 5;

/** 移動時間の基本係数（秒/マス）。SPEEDで割る */
export const BASE_TRAVEL_TIME_PER_TILE = 2.0;

/** 魔獣の基本ステータス定義 */
export interface CardStats {
  /** 魔獣数（HP/ベース係数） */
  monster_count: number;
  /** スピード（行動順序） */
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

/** 各キャラクターの固有ステータス */
const CHARACTER_STATS: Record<number, Partial<CardStats>> = {
  // === プレイヤー初期魔獣（KC 7種族） ===
  0: { monster_count: 15, attack: 9, intelligence: 3, defense: 4, magicDefense: 3, speed: 6 },  // ダイアウルフ
  1: { monster_count: 18, attack: 7, intelligence: 4, defense: 6, magicDefense: 3, speed: 5 },  // ゴブリンウォリアー
  2: { monster_count: 10, attack: 4, intelligence: 10, defense: 2, magicDefense: 6, speed: 7, range: 2 }, // インプ
  3: { monster_count: 12, attack: 8, intelligence: 6, defense: 5, magicDefense: 5, speed: 5, range: 2 },  // ワイバーン
  4: { monster_count: 16, attack: 6, intelligence: 2, defense: 10, magicDefense: 4, speed: 3 }, // ゴーレム
  5: { monster_count: 11, attack: 5, intelligence: 9, defense: 3, magicDefense: 7, speed: 5, range: 2 },  // サラマンダー
  6: { monster_count: 14, attack: 7, intelligence: 3, defense: 7, magicDefense: 4, speed: 4 },  // スケルトンソルジャー
  7: { monster_count: 13, attack: 8, intelligence: 4, defense: 3, magicDefense: 3, speed: 7 },  // ヘルハウンド
  8: { monster_count: 14, attack: 8, intelligence: 5, defense: 5, magicDefense: 4, speed: 5 },  // リザードマン
  9: { monster_count: 16, attack: 5, intelligence: 7, defense: 8, magicDefense: 6, speed: 3 },  // トレント

  // === フィールド敵（Lv1〜6） ===
  10: { monster_count: 2, attack: 3, intelligence: 2, defense: 2, magicDefense: 1, speed: 3 },  // ゴブリン
  11: { monster_count: 4, attack: 5, intelligence: 3, defense: 3, magicDefense: 2, speed: 4 },  // コボルド
  12: { monster_count: 6, attack: 8, intelligence: 4, defense: 6, magicDefense: 3, speed: 3 },  // オーク
  13: { monster_count: 8, attack: 10, intelligence: 6, defense: 8, magicDefense: 5, speed: 4 }, // スケルトン
  14: { monster_count: 12, attack: 15, intelligence: 3, defense: 12, magicDefense: 4, speed: 3 }, // トロール
  15: { monster_count: 15, attack: 20, intelligence: 15, defense: 15, magicDefense: 10, speed: 6, range: 2 }, // ドレイク

  // === 遺跡敵（ノーマル） ===
  20: { monster_count: 8, attack: 6, intelligence: 2, defense: 10, magicDefense: 4, speed: 2 },  // ストーンゴーレム
  21: { monster_count: 6, attack: 4, intelligence: 8, defense: 3, magicDefense: 8, speed: 5, range: 2 },   // ゴースト
  22: { monster_count: 10, attack: 10, intelligence: 3, defense: 8, magicDefense: 4, speed: 4 }, // スケルトンナイト
  23: { monster_count: 12, attack: 7, intelligence: 8, defense: 4, magicDefense: 6, speed: 5 }, // コカトリス
  24: { monster_count: 7, attack: 8, intelligence: 5, defense: 5, magicDefense: 5, speed: 7 },   // ミミック
  25: { monster_count: 5, attack: 6, intelligence: 4, defense: 3, magicDefense: 3, speed: 6 },   // ポイズンスパイダー

  // === 遺跡敵（レア） ===
  30: { monster_count: 12, attack: 12, intelligence: 10, defense: 6, magicDefense: 8, speed: 5, range: 2 }, // ダークウィザード
  31: { monster_count: 14, attack: 14, intelligence: 4, defense: 12, magicDefense: 6, speed: 4 }, // ガーゴイル
  32: { monster_count: 10, attack: 16, intelligence: 2, defense: 6, magicDefense: 4, speed: 8 },  // シャドウアサシン
  33: { monster_count: 9, attack: 6, intelligence: 14, defense: 4, magicDefense: 10, speed: 5, range: 2 },  // フレイムスピリット
  34: { monster_count: 9, attack: 6, intelligence: 14, defense: 4, magicDefense: 10, speed: 5, range: 2 },  // アイスエレメンタル
  35: { monster_count: 16, attack: 12, intelligence: 6, defense: 14, magicDefense: 8, speed: 4 }, // デスナイト
  36: { monster_count: 14, attack: 14, intelligence: 12, defense: 10, magicDefense: 10, speed: 4 }, // ヒュドラ
  37: { monster_count: 14, attack: 16, intelligence: 4, defense: 12, magicDefense: 6, speed: 5 }, // ミノタウロス

  // === 遺跡敵（レジェンダリー） ===
  40: { monster_count: 30, attack: 25, intelligence: 18, defense: 20, magicDefense: 15, speed: 4 }, // ニーズヘッグ
  41: { monster_count: 22, attack: 18, intelligence: 20, defense: 14, magicDefense: 18, speed: 6 }, // ヴァンパイアロード
  42: { monster_count: 22, attack: 15, intelligence: 25, defense: 12, magicDefense: 20, speed: 5, range: 2 }, // リッチ
  43: { monster_count: 35, attack: 22, intelligence: 12, defense: 25, magicDefense: 18, speed: 2 }, // タイタン

  // === 収集可能魔獣 ===
  // --- 獣族 ---
  50: { monster_count: 8, attack: 4, intelligence: 2, defense: 3, magicDefense: 2, speed: 8 },   // バット
  51: { monster_count: 10, attack: 6, intelligence: 3, defense: 5, magicDefense: 3, speed: 7 },  // ジャイアントバット
  52: { monster_count: 12, attack: 7, intelligence: 4, defense: 5, magicDefense: 4, speed: 7 },  // ヴァンパイアバット
  53: { monster_count: 14, attack: 9, intelligence: 5, defense: 6, magicDefense: 5, speed: 8, range: 2 },  // カマソッツ
  54: { monster_count: 11, attack: 7, intelligence: 4, defense: 6, magicDefense: 3, speed: 5 },  // ボーゲスト
  55: { monster_count: 14, attack: 10, intelligence: 5, defense: 7, magicDefense: 4, speed: 6 }, // ガイトラッシュ
  56: { monster_count: 18, attack: 14, intelligence: 6, defense: 8, magicDefense: 5, speed: 9 }, // レウクロコタ
  // --- 亜人族 ---
  60: { monster_count: 6, attack: 5, intelligence: 2, defense: 2, magicDefense: 1, speed: 4, range: 3, cost: 1.0 },   // ゴブリンアーチャー
  61: { monster_count: 10, attack: 5, intelligence: 5, defense: 5, magicDefense: 4, speed: 4, range: 2 },  // ゴブリンコック
  62: { monster_count: 12, attack: 7, intelligence: 5, defense: 8, magicDefense: 5, speed: 5, range: 2 },  // ホブゴブリン
  63: { monster_count: 14, attack: 8, intelligence: 4, defense: 10, magicDefense: 5, speed: 3 }, // オークアーマーナイト
  64: { monster_count: 12, attack: 9, intelligence: 5, defense: 6, magicDefense: 4, speed: 6 },  // ゴブリンソードマン
  65: { monster_count: 16, attack: 12, intelligence: 4, defense: 11, magicDefense: 6, speed: 5, range: 2 },// ホブゴブリンダークナイト
  66: { monster_count: 15, attack: 12, intelligence: 10, defense: 10, magicDefense: 8, speed: 5 },// ゴブリンプリンセス
  // --- 魔族 ---
  70: { monster_count: 8, attack: 6, intelligence: 5, defense: 4, magicDefense: 5, speed: 5 },   // レッサーデーモン
  71: { monster_count: 10, attack: 5, intelligence: 10, defense: 4, magicDefense: 8, speed: 6, range: 2 }, // サキュバス
  72: { monster_count: 12, attack: 8, intelligence: 6, defense: 6, magicDefense: 5, speed: 7 },  // ナイトメア
  73: { monster_count: 16, attack: 13, intelligence: 8, defense: 10, magicDefense: 8, speed: 5 },// アークデーモン
  74: { monster_count: 14, attack: 10, intelligence: 16, defense: 8, magicDefense: 14, speed: 7, range: 2 },// リリス
  // --- 竜族 ---
  80: { monster_count: 8, attack: 7, intelligence: 4, defense: 5, magicDefense: 4, speed: 4 },   // リンドヴルム
  81: { monster_count: 12, attack: 8, intelligence: 6, defense: 7, magicDefense: 6, speed: 5 },  // シーサーペント
  82: { monster_count: 14, attack: 11, intelligence: 9, defense: 8, magicDefense: 7, speed: 5, range: 2 }, // ファイアドレイク
  83: { monster_count: 20, attack: 18, intelligence: 14, defense: 16, magicDefense: 12, speed: 4, range: 2 },// バハムート
  // --- 巨人族 ---
  90: { monster_count: 10, attack: 7, intelligence: 2, defense: 6, magicDefense: 2, speed: 3 },  // オーガ
  91: { monster_count: 14, attack: 9, intelligence: 3, defense: 9, magicDefense: 4, speed: 3 },  // サイクロプス
  92: { monster_count: 18, attack: 10, intelligence: 3, defense: 14, magicDefense: 6, speed: 2 },// アイアンゴーレム
  93: { monster_count: 22, attack: 16, intelligence: 5, defense: 18, magicDefense: 10, speed: 3 },// ギガース
  // --- 精霊族 ---
  100: { monster_count: 6, attack: 3, intelligence: 7, defense: 2, magicDefense: 6, speed: 6, range: 2 },  // ウィスプ
  101: { monster_count: 8, attack: 4, intelligence: 9, defense: 3, magicDefense: 7, speed: 7, range: 2 },  // シルフ
  102: { monster_count: 10, attack: 5, intelligence: 10, defense: 5, magicDefense: 9, speed: 5, range: 2 },// ウンディーネ
  103: { monster_count: 14, attack: 12, intelligence: 11, defense: 7, magicDefense: 8, speed: 5, range: 2 },// イフリート
  104: { monster_count: 16, attack: 10, intelligence: 15, defense: 8, magicDefense: 14, speed: 6, range: 2 },// フェニックス
  // --- 不死族 ---
  110: { monster_count: 10, attack: 5, intelligence: 1, defense: 6, magicDefense: 2, speed: 2 }, // ゾンビ
  111: { monster_count: 8, attack: 4, intelligence: 8, defense: 3, magicDefense: 9, speed: 5, range: 2 },  // レイス
  112: { monster_count: 12, attack: 8, intelligence: 5, defense: 7, magicDefense: 5, speed: 4 }, // ワイト
  113: { monster_count: 14, attack: 11, intelligence: 4, defense: 10, magicDefense: 6, speed: 5 },// ドゥラハン
  114: { monster_count: 16, attack: 8, intelligence: 18, defense: 8, magicDefense: 16, speed: 4, range: 2, cost: 1.0 },// エルダーリッチ
};

/** キャラクターの完全ステータスを取得 */
export function getCharacterStats(index: number): CardStats {
  const custom = CHARACTER_STATS[index] ?? {};
  return { ...DEFAULT_CARD_STATS, ...custom };
}

/** 全キャラクター名 */
const CHARACTER_NAMES: Record<number, string> = {
  // プレイヤー初期魔獣（KC 7種族）
  0: "ダイアウルフ",
  1: "ゴブリンウォリアー",
  2: "インプ",
  3: "ワイバーン",
  4: "ゴーレム",
  5: "サラマンダー",
  6: "スケルトンソルジャー",
  7: "ヘルハウンド",
  8: "リザードマン",
  9: "トレント",
  // フィールド敵
  10: "ゴブリン",
  11: "コボルド",
  12: "オーク",
  13: "スケルトン",
  14: "トロール",
  15: "ドレイク",
  // 遺跡敵（ノーマル）
  20: "ストーンゴーレム",
  21: "ゴースト",
  22: "スケルトンナイト",
  23: "コカトリス",
  24: "ミミック",
  25: "ポイズンスパイダー",
  // 遺跡敵（レア）
  30: "ダークウィザード",
  31: "ガーゴイル",
  32: "シャドウアサシン",
  33: "フレイムスピリット",
  34: "アイスエレメンタル",
  35: "デスナイト",
  36: "ヒュドラ",
  37: "ミノタウロス",
  // 遺跡敵（レジェンダリー）
  40: "ニーズヘッグ",
  41: "ヴァンパイアロード",
  42: "リッチ",
  43: "タイタン",
  // 獣族
  50: "バット", 51: "ジャイアントバット", 52: "ヴァンパイアバット",
  53: "カマソッツ", 54: "ボーゲスト", 55: "ガイトラッシュ", 56: "レウクロコタ",
  // 亜人族
  60: "ゴブリンアーチャー", 61: "ゴブリンコック", 62: "ホブゴブリン",
  63: "オークアーマーナイト", 64: "ゴブリンソードマン", 65: "ホブゴブリンダークナイト", 66: "ゴブリンプリンセス",
  // 魔族
  70: "レッサーデーモン", 71: "サキュバス", 72: "ナイトメア", 73: "アークデーモン", 74: "リリス",
  // 竜族
  80: "リンドヴルム", 81: "シーサーペント", 82: "ファイアドレイク", 83: "バハムート",
  // 巨人族
  90: "オーガ", 91: "サイクロプス", 92: "アイアンゴーレム", 93: "ギガース",
  // 精霊族
  100: "ウィスプ", 101: "シルフ", 102: "ウンディーネ", 103: "イフリート", 104: "フェニックス",
  // 不死族
  110: "ゾンビ", 111: "レイス", 112: "ワイト", 113: "ドゥラハン", 114: "エルダーリッチ",
};

/** 魔獣のレアリティ */
const CHARACTER_RARITY: Record<number, CardRarity> = {
  // プレイヤー初期魔獣
  0: "rare", 1: "rare", 2: "rare", 3: "rare", 4: "rare",
  5: "rare", 6: "rare", 7: "rare", 8: "rare", 9: "rare",
  // フィールド敵
  10: "common", 11: "common", 12: "uncommon", 13: "uncommon", 14: "rare", 15: "epic",
  // 遺跡敵（ノーマル）
  20: "uncommon", 21: "uncommon", 22: "uncommon", 23: "rare", 24: "rare", 25: "uncommon",
  // 遺跡敵（レア）
  30: "rare", 31: "rare", 32: "rare", 33: "rare", 34: "rare", 35: "epic", 36: "epic", 37: "epic",
  // 遺跡敵（レジェンダリー）
  40: "legendary", 41: "legendary", 42: "legendary", 43: "legendary",
  // 獣族
  50: "common", 51: "uncommon", 52: "uncommon", 53: "rare", 54: "uncommon", 55: "rare", 56: "epic",
  // 亜人族
  60: "common", 61: "common", 62: "uncommon", 63: "uncommon", 64: "uncommon", 65: "rare", 66: "epic",
  // 魔族
  70: "common", 71: "uncommon", 72: "uncommon", 73: "rare", 74: "epic",
  // 竜族
  80: "common", 81: "uncommon", 82: "rare", 83: "epic",
  // 巨人族
  90: "common", 91: "uncommon", 92: "rare", 93: "epic",
  // 精霊族
  100: "common", 101: "uncommon", 102: "uncommon", 103: "rare", 104: "epic",
  // 不死族
  110: "common", 111: "uncommon", 112: "uncommon", 113: "rare", 114: "epic",
};

/** キャラクターの種族 */
const CHARACTER_RACES: Record<number, Race> = {
  0: "beast", 1: "demihuman", 2: "demon", 3: "dragon", 4: "giant",
  5: "spirit", 6: "undead", 7: "beast", 8: "demihuman", 9: "spirit",
  10: "demihuman", 11: "demihuman", 12: "demihuman", 13: "undead", 14: "giant", 15: "dragon",
  20: "giant", 21: "undead", 22: "undead", 23: "beast", 24: "demon", 25: "beast",
  30: "demon", 31: "demon", 32: "demon", 33: "spirit", 34: "spirit", 35: "undead", 36: "dragon", 37: "giant",
  40: "dragon", 41: "undead", 42: "undead", 43: "giant",
  // 獣族
  50: "beast", 51: "beast", 52: "beast", 53: "beast", 54: "beast", 55: "beast", 56: "beast",
  // 亜人族
  60: "demihuman", 61: "demihuman", 62: "demihuman", 63: "demihuman", 64: "demihuman", 65: "demihuman", 66: "demihuman",
  // 魔族
  70: "demon", 71: "demon", 72: "demon", 73: "demon", 74: "demon",
  // 竜族
  80: "dragon", 81: "dragon", 82: "dragon", 83: "dragon",
  // 巨人族
  90: "giant", 91: "giant", 92: "giant", 93: "giant",
  // 精霊族
  100: "spirit", 101: "spirit", 102: "spirit", 103: "spirit", 104: "spirit",
  // 不死族
  110: "undead", 111: "undead", 112: "undead", 113: "undead", 114: "undead",
};

export function getBodyDisplayName(index: number): string {
  return CHARACTER_NAMES[index] ?? `キャラ${index + 1}`;
}

/** 魔獣イラストのパス（キャラクターインデックス→画像URL） */
export function getCharacterIllustrationPath(index: number): string {
  return `/cards/character-${index}.png`;
}

/** 魔獣のレアリティを取得 */
export function getCardRarity(index: number): CardRarity {
  return CHARACTER_RARITY[index] ?? "common";
}

/** 魔獣の種族を取得 */
export function getCardRace(index: number): Race {
  return CHARACTER_RACES[index] ?? "demihuman";
}

/** 魔獣名からインデックスを取得 */
export function getCardIndexByName(name: string): number | undefined {
  for (const [idx, n] of Object.entries(CHARACTER_NAMES)) {
    if (n === name) return parseInt(idx);
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
