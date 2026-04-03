/**
 * 遺跡限定敵キャラクター定義
 */

import type { SkillDataPayload } from "../shared/game-state";

/** 遺跡敵のタイプ */
export type RuinEnemyType = 
  // 基本敵
  | "golem" 
  | "phantom" 
  | "skeleton_knight" 
  | "treasure_mimic" 
  | "dark_wizard" 
  // 追加敵
  | "slime_king"
  | "cursed_armor"
  | "shadow_assassin"
  | "flame_spirit"
  | "ice_elemental"
  | "poison_spider"
  | "stone_gargoyle"
  | "death_knight"
  | "necromancer"
  | "crystal_golem"
  | "thunder_hawk"
  | "earth_wyrm"
  | "void_stalker"
  | "ancient_mummy"
  | "demon_imp"
  // ボス敵
  | "ruin_guardian"
  | "dragon_zombie"
  | "lich_lord"
  | "titan_colossus";

/** 遺跡敵キャラクター定義 */
export interface RuinEnemyDef {
  type: RuinEnemyType;
  name: string;
  monster_count: number;
  speed: number;
  icon: string;
  description: string;
  skills: SkillDataPayload;
}

/** 全遺跡敵定義 */
export const RUIN_ENEMIES: Record<RuinEnemyType, RuinEnemyDef> = {
  // === 基本敵 ===
  golem: {
    type: "golem",
    name: "ゴーレム",
    monster_count: 15,
    speed: 3,
    icon: "🗿",
    description: "岩の守り: 防御+30%",
    skills: { passive_id: "stone_guard", active_id: "heavy_strike" },
  },
  phantom: {
    type: "phantom",
    name: "ゴースト",
    monster_count: 8,
    speed: 8,
    icon: "👻",
    description: "幽体: 回避率+25%",
    skills: { passive_id: "ethereal", active_id: "soul_drain" },
  },
  skeleton_knight: {
    type: "skeleton_knight",
    name: "スケルトンナイト",
    monster_count: 10,
    speed: 5,
    icon: "💀",
    description: "骨の反撃: 被ダメ時反射10%",
    skills: { passive_id: "bone_counter", active_id: "sword_slash" },
  },
  treasure_mimic: {
    type: "treasure_mimic",
    name: "ミミック",
    monster_count: 5,
    speed: 6,
    icon: "📦",
    description: "お宝持ち: ドロップ率2倍",
    skills: { passive_id: "treasure_bearer", active_id: "surprise_bite" },
  },
  dark_wizard: {
    type: "dark_wizard",
    name: "ダークウィザード",
    monster_count: 12,
    speed: 4,
    icon: "🧙",
    description: "闇の呪い: 攻撃時毒付与",
    skills: { passive_id: "dark_curse", active_id: "shadow_bolt" },
  },

  // === 追加敵（低〜中級） ===
  slime_king: {
    type: "slime_king",
    name: "コカトリス",
    monster_count: 20,
    speed: 5,
    icon: "🐓",
    description: "石化の眼光: 攻撃時石化付与",
    skills: { passive_id: "split_on_death", active_id: "acid_splash" },
  },
  cursed_armor: {
    type: "cursed_armor",
    name: "ガーゴイル",
    monster_count: 14,
    speed: 4,
    icon: "🦇",
    description: "石像の守護: ダメージ軽減+物理反射",
    skills: { passive_id: "cursed_shell", active_id: "cursed_slash" },
  },
  shadow_assassin: {
    type: "shadow_assassin",
    name: "シャドウアサシン",
    monster_count: 7,
    speed: 10,
    icon: "🗡️",
    description: "暗殺者: クリティカル率+50%",
    skills: { passive_id: "deadly_precision", active_id: "backstab" },
  },
  flame_spirit: {
    type: "flame_spirit",
    name: "フレイムスピリット",
    monster_count: 9,
    speed: 7,
    icon: "🔥",
    description: "炎の精霊: 攻撃時炎上付与",
    skills: { passive_id: "burning_aura", active_id: "fireball" },
  },
  ice_elemental: {
    type: "ice_elemental",
    name: "アイスエレメンタル",
    monster_count: 11,
    speed: 5,
    icon: "❄️",
    description: "氷の精霊: 攻撃時凍結付与",
    skills: { passive_id: "freezing_touch", active_id: "ice_spike" },
  },
  poison_spider: {
    type: "poison_spider",
    name: "ポイズンスパイダー",
    monster_count: 6,
    speed: 7,
    icon: "🕷️",
    description: "猛毒の牙: 攻撃時強毒付与",
    skills: { passive_id: "venom_fangs", active_id: "poison_bite" },
  },
  stone_gargoyle: {
    type: "stone_gargoyle",
    name: "ストーンゴーレム",
    monster_count: 13,
    speed: 6,
    icon: "🦇",
    description: "石化防御: 被ダメ時一定確率で無効化",
    skills: { passive_id: "stone_skin", active_id: "diving_strike" },
  },

  // === 追加敵（中〜上級） ===
  death_knight: {
    type: "death_knight",
    name: "デスナイト",
    monster_count: 18,
    speed: 4,
    icon: "⚔️",
    description: "死の騎士: 敵撃破時HP回復",
    skills: { passive_id: "soul_harvest", active_id: "death_strike" },
  },
  necromancer: {
    type: "necromancer",
    name: "ネクロマンサー",
    monster_count: 10,
    speed: 3,
    icon: "☠️",
    description: "死霊術師: 味方死亡時強化",
    skills: { passive_id: "dark_ritual", active_id: "raise_dead" },
  },
  crystal_golem: {
    type: "crystal_golem",
    name: "ミノタウロス",
    monster_count: 22,
    speed: 4,
    icon: "🐂",
    description: "猛牛の力: 高攻撃+突進攻撃",
    skills: { passive_id: "crystal_armor", active_id: "crystal_smash" },
  },
  thunder_hawk: {
    type: "thunder_hawk",
    name: "サンダーホーク",
    monster_count: 8,
    speed: 12,
    icon: "🦅",
    description: "雷鳥: 超高速+感電付与",
    skills: { passive_id: "lightning_speed", active_id: "thunder_dive" },
  },
  earth_wyrm: {
    type: "earth_wyrm",
    name: "アースワーム",
    monster_count: 16,
    speed: 3,
    icon: "🐛",
    description: "大地の蟲: 地震攻撃+高耐久",
    skills: { passive_id: "earth_armor", active_id: "earthquake" },
  },
  void_stalker: {
    type: "void_stalker",
    name: "ヴォイドストーカー",
    monster_count: 12,
    speed: 9,
    icon: "👁️",
    description: "虚空の狩人: 沈黙付与+回避",
    skills: { passive_id: "void_cloak", active_id: "silence_strike" },
  },
  ancient_mummy: {
    type: "ancient_mummy",
    name: "エンシェントマミー",
    monster_count: 14,
    speed: 2,
    icon: "🧟",
    description: "古代のミイラ: 呪い+自己回復",
    skills: { passive_id: "ancient_curse", active_id: "bandage_strangle" },
  },
  demon_imp: {
    type: "demon_imp",
    name: "デーモンインプ",
    monster_count: 6,
    speed: 8,
    icon: "😈",
    description: "小悪魔: 味方強化+トリッキー",
    skills: { passive_id: "demonic_aura", active_id: "chaos_bolt" },
  },

  // === ボス敵 ===
  ruin_guardian: {
    type: "ruin_guardian",
    name: "ニーズヘッグ",
    monster_count: 30,
    speed: 4,
    icon: "🐉",
    description: "冥界の龍: 猛毒ブレス+高耐久",
    skills: { passive_id: "ancient_power", active_id: "devastating_blow", unique_id: "guardian_wrath" },
  },
  dragon_zombie: {
    type: "dragon_zombie",
    name: "ヴァンパイアロード",
    monster_count: 22,
    speed: 6,
    icon: "🧛",
    description: "吸血貴族: 攻撃時HP吸収+魅了",
    skills: { passive_id: "undead_dragon", active_id: "death_breath", unique_id: "resurrection" },
  },
  lich_lord: {
    type: "lich_lord",
    name: "リッチ",
    monster_count: 22,
    speed: 5,
    icon: "💀",
    description: "死霊術の極致: 全体呪い+即死攻撃",
    skills: { passive_id: "lord_of_undead", active_id: "soul_rend", unique_id: "death_sentence" },
  },
  titan_colossus: {
    type: "titan_colossus",
    name: "タイタン",
    monster_count: 35,
    speed: 2,
    icon: "⛰️",
    description: "巨人族の王: 超高耐久+全体攻撃",
    skills: { passive_id: "titan_body", active_id: "colossal_strike", unique_id: "apocalypse" },
  },
};

/** 遺跡の難易度 */
export type RuinDifficulty = "normal" | "rare" | "legendary";

/** 遺跡ユニット編成 */
export interface RuinUnitFormation {
  name: string;
  difficulty: RuinDifficulty;
  enemies: [RuinEnemyType, RuinEnemyType, RuinEnemyType];
}

/** 遺跡ユニット編成パターン */
export const RUIN_FORMATIONS: RuinUnitFormation[] = [
  // === ノーマル遺跡（難易度: ★★☆） ===
  { name: "石の番人", difficulty: "normal", enemies: ["golem", "golem", "golem"] },
  { name: "亡霊の群れ", difficulty: "normal", enemies: ["phantom", "phantom", "phantom"] },
  { name: "骸骨兵団", difficulty: "normal", enemies: ["skeleton_knight", "skeleton_knight", "skeleton_knight"] },
  { name: "スライムの巣", difficulty: "normal", enemies: ["slime_king", "treasure_mimic", "treasure_mimic"] },
  { name: "蜘蛛の巣窟", difficulty: "normal", enemies: ["poison_spider", "poison_spider", "poison_spider"] },
  { name: "炎の回廊", difficulty: "normal", enemies: ["flame_spirit", "flame_spirit", "golem"] },
  { name: "氷結の間", difficulty: "normal", enemies: ["ice_elemental", "ice_elemental", "phantom"] },
  { name: "小悪魔の遊び場", difficulty: "normal", enemies: ["demon_imp", "demon_imp", "demon_imp"] },
  
  // === レア遺跡（難易度: ★★★） ===
  { name: "闇の魔術師団", difficulty: "rare", enemies: ["dark_wizard", "dark_wizard", "dark_wizard"] },
  { name: "宝箱の罠", difficulty: "rare", enemies: ["treasure_mimic", "treasure_mimic", "treasure_mimic"] },
  { name: "混成警備隊", difficulty: "rare", enemies: ["golem", "skeleton_knight", "phantom"] },
  { name: "闇と骨の同盟", difficulty: "rare", enemies: ["dark_wizard", "skeleton_knight", "skeleton_knight"] },
  { name: "呪われた武具庫", difficulty: "rare", enemies: ["cursed_armor", "cursed_armor", "cursed_armor"] },
  { name: "暗殺者の隠れ家", difficulty: "rare", enemies: ["shadow_assassin", "shadow_assassin", "void_stalker"] },
  { name: "精霊の聖域", difficulty: "rare", enemies: ["flame_spirit", "ice_elemental", "thunder_hawk"] },
  { name: "ガーゴイルの塔", difficulty: "rare", enemies: ["stone_gargoyle", "stone_gargoyle", "stone_gargoyle"] },
  { name: "死者の墓所", difficulty: "rare", enemies: ["ancient_mummy", "skeleton_knight", "necromancer"] },
  { name: "クリスタルの洞窟", difficulty: "rare", enemies: ["crystal_golem", "golem", "golem"] },
  { name: "雷鳥の巣", difficulty: "rare", enemies: ["thunder_hawk", "thunder_hawk", "thunder_hawk"] },
  { name: "地底の主", difficulty: "rare", enemies: ["earth_wyrm", "poison_spider", "poison_spider"] },
  
  // === ボス遺跡（難易度: ★★★★★） ===
  { name: "守護者の間", difficulty: "legendary", enemies: ["ruin_guardian", "golem", "golem"] },
  { name: "暗黒の祭壇", difficulty: "legendary", enemies: ["dark_wizard", "dark_wizard", "ruin_guardian"] },
  { name: "最深部の番人", difficulty: "legendary", enemies: ["ruin_guardian", "ruin_guardian", "ruin_guardian"] },
  { name: "竜の墓場", difficulty: "legendary", enemies: ["dragon_zombie", "death_knight", "death_knight"] },
  { name: "死霊王の玉座", difficulty: "legendary", enemies: ["lich_lord", "necromancer", "necromancer"] },
  { name: "巨神の神殿", difficulty: "legendary", enemies: ["titan_colossus", "crystal_golem", "crystal_golem"] },
  { name: "混沌の深淵", difficulty: "legendary", enemies: ["void_stalker", "void_stalker", "lich_lord"] },
  { name: "不死の軍団", difficulty: "legendary", enemies: ["dragon_zombie", "lich_lord", "death_knight"] },
  { name: "終焉の間", difficulty: "legendary", enemies: ["titan_colossus", "dragon_zombie", "lich_lord"] },
];

/** 難易度に応じた編成をランダムに取得 */
export function getRandomFormation(difficulty?: RuinDifficulty): RuinUnitFormation {
  const filtered = difficulty 
    ? RUIN_FORMATIONS.filter(f => f.difficulty === difficulty)
    : RUIN_FORMATIONS;
  return filtered[Math.floor(Math.random() * filtered.length)];
}

/** 編成から戦闘用データを生成 */
export function getFormationBattleData(formation: RuinUnitFormation): {
  names: string[];
  monster_counts: number[];
  speeds: number[];
  skills: SkillDataPayload[];
  enemyTypes: RuinEnemyType[];
} {
  const names: string[] = [];
  const monster_counts: number[] = [];
  const speeds: number[] = [];
  const skills: SkillDataPayload[] = [];
  const enemyTypes: RuinEnemyType[] = [];
  
  for (const enemyType of formation.enemies) {
    const enemy = RUIN_ENEMIES[enemyType];
    names.push(enemy.name);
    monster_counts.push(enemy.monster_count);
    speeds.push(enemy.speed);
    skills.push(enemy.skills);
    enemyTypes.push(enemyType);
  }
  
  return { names, monster_counts, speeds, skills, enemyTypes };
}

/** 難易度の表示名 */
export function getDifficultyLabel(difficulty: RuinDifficulty): string {
  switch (difficulty) {
    case "normal": return "★";
    case "rare": return "★★";
    case "legendary": return "★★★";
  }
}

/** 難易度の色 */
export function getDifficultyColor(difficulty: RuinDifficulty): string {
  switch (difficulty) {
    case "normal": return "#5da845";
    case "rare": return "#c9a84c";
    case "legendary": return "#c0392b";
  }
}
