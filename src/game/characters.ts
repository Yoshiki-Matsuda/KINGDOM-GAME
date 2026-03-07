import { getCharacterSkills, type CharacterSkills, type SkillData, getCharacterSkillData } from "./skills";

/** 3体で1ユニットが編成される */
export const BODIES_PER_UNIT = 3;

/** キャラ1体あたりの仮のエナジー（必ず勝てる数値） */
export const DEFAULT_BODY_ENERGY = 10;

/** キャラ1体あたりのデフォルトSPEED（1〜10、高いほど速い） */
export const DEFAULT_BODY_SPEED = 5;

/** 移動時間の基本係数（秒/マス）。SPEEDで割る */
export const BASE_TRAVEL_TIME_PER_TILE = 2.0;

/** カードの基本ステータス定義 */
export interface CardStats {
  /** エナジー（HP/ベース係数） */
  energy: number;
  /** スピード（行動順序） */
  speed: number;
  /** 攻撃力（物理ダメージ） */
  attack: number;
  /** 魔力（魔法ダメージ） */
  magic: number;
  /** 防御力（物理防御） */
  defense: number;
  /** 魔法防御力 */
  magicDefense: number;
}

/** デフォルトのカードステータス */
export const DEFAULT_CARD_STATS: CardStats = {
  energy: 10,
  speed: 5,
  attack: 5,
  magic: 5,
  defense: 3,
  magicDefense: 3,
};

/** キャラクターデータ */
export interface CharacterData {
  index: number;
  name: string;
  stats: CardStats;
  skills: CharacterSkills;
}

/** カードのレアリティ */
export type CardRarity = "common" | "uncommon" | "rare" | "epic" | "legendary";

/** 各キャラクターの固有ステータス */
const CHARACTER_STATS: Record<number, Partial<CardStats>> = {
  // === プレイヤー初期カード（北欧神話） ===
  0: { energy: 15, attack: 8, magic: 3, defense: 5, magicDefense: 3, speed: 4 }, // オーディン: バランス型
  1: { energy: 18, attack: 12, magic: 2, defense: 6, magicDefense: 2, speed: 3 }, // トール: 物理アタッカー
  2: { energy: 10, attack: 4, magic: 10, defense: 2, magicDefense: 5, speed: 7 }, // ロキ: 魔法アタッカー
  3: { energy: 12, attack: 3, magic: 9, defense: 3, magicDefense: 7, speed: 5 },  // フレイヤ: 魔法サポート
  4: { energy: 14, attack: 7, magic: 6, defense: 4, magicDefense: 4, speed: 5 },  // フレイ: ハイブリッド
  5: { energy: 16, attack: 5, magic: 3, defense: 8, magicDefense: 6, speed: 4 },  // ヘイムダル: タンク
  6: { energy: 13, attack: 6, magic: 7, defense: 4, magicDefense: 5, speed: 5 },  // バルドル: バランス型
  7: { energy: 14, attack: 9, magic: 2, defense: 5, magicDefense: 2, speed: 6 },  // ティール: 物理アタッカー
  8: { energy: 12, attack: 4, magic: 8, defense: 3, magicDefense: 6, speed: 5 },  // ニョルド: 魔法型
  9: { energy: 11, attack: 7, magic: 4, defense: 3, magicDefense: 3, speed: 8 },  // ウール: スピード型
  
  // === フィールド敵（Lv1〜6） ===
  10: { energy: 2, attack: 3, magic: 2, defense: 2, magicDefense: 1, speed: 3 },  // スライム: 最弱
  11: { energy: 4, attack: 5, magic: 3, defense: 3, magicDefense: 2, speed: 4 },  // ゴブリン: 雑魚
  12: { energy: 6, attack: 8, magic: 4, defense: 6, magicDefense: 3, speed: 3 },  // オーク: 物理型
  13: { energy: 8, attack: 10, magic: 6, defense: 8, magicDefense: 5, speed: 4 }, // 骸骨戦士: バランス
  14: { energy: 12, attack: 15, magic: 8, defense: 12, magicDefense: 6, speed: 3 }, // オーガ: 重量級
  15: { energy: 15, attack: 20, magic: 15, defense: 15, magicDefense: 10, speed: 6 }, // ワイバーン: 強敵

  // === 遺跡敵（ノーマル） ===
  20: { energy: 8, attack: 6, magic: 2, defense: 10, magicDefense: 4, speed: 2 },  // ゴーレム: 防御型
  21: { energy: 6, attack: 4, magic: 8, defense: 3, magicDefense: 8, speed: 5 },   // ファントム: 魔法型
  22: { energy: 10, attack: 10, magic: 3, defense: 8, magicDefense: 4, speed: 4 }, // スケルトンナイト: 物理
  23: { energy: 12, attack: 5, magic: 12, defense: 4, magicDefense: 6, speed: 4 }, // スライムキング: 魔法
  24: { energy: 7, attack: 8, magic: 5, defense: 5, magicDefense: 5, speed: 7 },   // トレジャーミミック: バランス
  25: { energy: 5, attack: 6, magic: 4, defense: 3, magicDefense: 3, speed: 6 },   // 毒蜘蛛: 速攻

  // === 遺跡敵（レア） ===
  30: { energy: 12, attack: 12, magic: 10, defense: 6, magicDefense: 8, speed: 5 }, // ダークウィザード: 魔法
  31: { energy: 14, attack: 14, magic: 4, defense: 12, magicDefense: 6, speed: 3 }, // 呪われた鎧: 物理防御
  32: { energy: 10, attack: 16, magic: 2, defense: 6, magicDefense: 4, speed: 8 },  // シャドウアサシン: 速攻
  33: { energy: 9, attack: 6, magic: 14, defense: 4, magicDefense: 10, speed: 5 },  // 炎の精霊: 魔法
  34: { energy: 9, attack: 6, magic: 14, defense: 4, magicDefense: 10, speed: 5 },  // 氷の精霊: 魔法
  35: { energy: 16, attack: 12, magic: 6, defense: 14, magicDefense: 8, speed: 4 }, // デスナイト: タンク
  36: { energy: 11, attack: 8, magic: 16, defense: 5, magicDefense: 12, speed: 5 }, // ネクロマンサー: 魔法
  37: { energy: 14, attack: 10, magic: 12, defense: 12, magicDefense: 12, speed: 4 }, // クリスタルゴーレム: バランス

  // === 遺跡敵（レジェンダリー） ===
  40: { energy: 25, attack: 18, magic: 10, defense: 20, magicDefense: 15, speed: 3 }, // 遺跡の守護者: 超タンク
  41: { energy: 30, attack: 25, magic: 15, defense: 18, magicDefense: 12, speed: 4 }, // ドラゴンゾンビ: 物理
  42: { energy: 22, attack: 15, magic: 25, defense: 12, magicDefense: 20, speed: 5 }, // リッチロード: 魔法
  43: { energy: 35, attack: 22, magic: 12, defense: 25, magicDefense: 18, speed: 2 }, // タイタンコロッサス: 超重量
};

/** キャラクターの完全ステータスを取得 */
export function getCharacterStats(index: number): CardStats {
  const custom = CHARACTER_STATS[index] ?? {};
  return { ...DEFAULT_CARD_STATS, ...custom };
}

/** 全キャラクター名 */
const CHARACTER_NAMES: Record<number, string> = {
  // プレイヤー初期カード（北欧神話）
  0: "オーディン",
  1: "トール",
  2: "ロキ",
  3: "フレイヤ",
  4: "フレイ",
  5: "ヘイムダル",
  6: "バルドル",
  7: "ティール",
  8: "ニョルド",
  9: "ウール",
  // フィールド敵
  10: "スライム",
  11: "ゴブリン",
  12: "オーク",
  13: "骸骨戦士",
  14: "オーガ",
  15: "ワイバーン",
  // 遺跡敵（ノーマル）
  20: "ゴーレム",
  21: "ファントム",
  22: "スケルトンナイト",
  23: "スライムキング",
  24: "トレジャーミミック",
  25: "毒蜘蛛",
  // 遺跡敵（レア）
  30: "ダークウィザード",
  31: "呪われた鎧",
  32: "シャドウアサシン",
  33: "炎の精霊",
  34: "氷の精霊",
  35: "デスナイト",
  36: "ネクロマンサー",
  37: "クリスタルゴーレム",
  // 遺跡敵（レジェンダリー）
  40: "遺跡の守護者",
  41: "ドラゴンゾンビ",
  42: "リッチロード",
  43: "タイタンコロッサス",
};

/** カードのレアリティ */
const CHARACTER_RARITY: Record<number, CardRarity> = {
  // プレイヤー初期カード
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
};

export function getBodyDisplayName(index: number): string {
  return CHARACTER_NAMES[index] ?? `キャラ${index + 1}`;
}

/** カードのレアリティを取得 */
export function getCardRarity(index: number): CardRarity {
  return CHARACTER_RARITY[index] ?? "common";
}

/** カード名からインデックスを取得 */
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
    stats: getCharacterStats(index),
    skills: getCharacterSkills(index),
  };
}

/** 後方互換: energy, speedを指定してデータ取得（既存コードとの互換用） */
export function getCharacterDataLegacy(index: number, energy: number, speed: number): CharacterData {
  const stats = getCharacterStats(index);
  return {
    index,
    name: getBodyDisplayName(index),
    stats: { ...stats, energy, speed },
    skills: getCharacterSkills(index),
  };
}

/** キャラクターのスキルデータを取得（サーバー送信用） */
export { getCharacterSkillData };
export type { SkillData };
