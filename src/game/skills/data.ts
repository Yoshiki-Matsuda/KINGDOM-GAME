/**
 * スキルマスターデータ — キャラごとのパッシブ・アクティブ・ユニークスキル
 */

import type { CharacterSkills, Skill } from "./types";

/** パッシブスキル一覧 */
export const PASSIVE_SKILLS: Record<string, Skill> = {
  // === 魔獣数・ステータス系 ===
  power_aura: {
    id: "power_aura",
    name: "闘気の波動",
    description: "戦闘開始時、味方全員の魔獣数1.15倍",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "monster_multiply", target: "ally_unit", value: 1.15 }],
  },
  wind_blessing: {
    id: "wind_blessing",
    name: "疾風の祝福",
    description: "戦闘開始時、味方全員のSPEED+2",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "speed_add", target: "ally_unit", value: 2 }],
  },
  life_blessing: {
    id: "life_blessing",
    name: "生命の恵み",
    description: "戦闘開始時、味方全員の魔獣数+3",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "monster_add", target: "ally_unit", value: 3 }],
  },
  rage_aura: {
    id: "rage_aura",
    name: "猛攻の気迫",
    description: "戦闘開始時、ダメージ1.2倍（被ダメも1.15倍）",
    category: "passive",
    timing: "battle_start",
    effects: [
      { type: "damage_multiply", target: "ally_unit", value: 1.2 },
      { type: "damage_reduce", target: "ally_unit", value: -0.15 },
    ],
  },

  // === 防御・軽減系 ===
  steel_guard: {
    id: "steel_guard",
    name: "鋼の守り",
    description: "戦闘開始時、被ダメージ20%軽減",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "damage_reduce", target: "self", value: 0.2 }],
  },
  iron_fortress: {
    id: "iron_fortress",
    name: "鉄壁の陣",
    description: "戦闘開始時、味方全員の被ダメージ10%軽減",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "damage_reduce", target: "ally_unit", value: 0.1 }],
  },
  barrier_field: {
    id: "barrier_field",
    name: "結界展開",
    description: "戦闘開始時、味方全員にシールド50付与（スキルLvで強化）",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "shield", target: "ally_unit", value: 50 }],
  },

  // === 反射・反撃系 ===
  thorns: {
    id: "thorns",
    name: "荊の鎧",
    description: "戦闘開始時、受けたダメージの20%を反射",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "damage_reflect", target: "self", value: 0.2 }],
  },
  counter_stance: {
    id: "counter_stance",
    name: "反撃の構え",
    description: "戦闘開始時、反撃態勢（50%で反撃）",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "counter", target: "self", value: 0.5 }],
  },

  // === 回避・隠密系 ===
  shadow_veil: {
    id: "shadow_veil",
    name: "影の帳",
    description: "戦闘開始時、回避率+20%",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "evasion", target: "self", value: 0.2 }],
  },
  mist_cloak: {
    id: "mist_cloak",
    name: "霧隠れ",
    description: "戦闘開始時、隠密状態（最初の攻撃を回避）",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "stealth", target: "self", value: 1 }],
  },

  // === 状態異常付与系 ===
  poison_aura: {
    id: "poison_aura",
    name: "瘴気の纏い",
    description: "戦闘開始時、敵全員に毒（毎ターン20ダメージ、スキルLvで強化）",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "poison", target: "enemy_all", value: 20, duration: { turns: 3 } }],
  },
  freezing_presence: {
    id: "freezing_presence",
    name: "凍てつく威圧",
    description: "戦闘開始時、30%で敵全員を凍結",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "freeze", target: "enemy_all", value: 1, duration: { turns: 1 } }],
    probability: 30,
  },
  intimidate: {
    id: "intimidate",
    name: "威圧",
    description: "戦闘開始時、敵全員のダメージ15%低下",
    category: "passive",
    timing: "battle_start",
    effects: [{ type: "weaken", target: "enemy_all", value: 0.15, duration: { turns: 2 } }],
  },

  // === 特殊系 ===
  last_stand: {
    id: "last_stand",
    name: "背水の陣",
    description: "HP50%以下で攻撃力2倍",
    category: "passive",
    timing: "hp_low",
    effects: [{ type: "damage_multiply", target: "self", value: 2.0, condition: { type: "hp_below", value: 50 } }],
  },
  first_strike: {
    id: "first_strike",
    name: "先制の心得",
    description: "初回攻撃時、ダメージ1.5倍",
    category: "passive",
    timing: "first_attack",
    effects: [{ type: "damage_multiply", target: "self", value: 1.5 }],
  },
  undying_will: {
    id: "undying_will",
    name: "不屈の意志",
    description: "致死ダメージを1回だけHP1で耐える",
    category: "passive",
    timing: "on_death",
    effects: [{ type: "monster_set", target: "self", value: 1 }],
    probability: 100,
  },
};

/** アクティブスキル一覧 */
export const ACTIVE_SKILLS: Record<string, Skill> = {
  // === ダメージ強化系 ===
  critical_edge: {
    id: "critical_edge",
    name: "会心撃",
    description: "攻撃時、25%でダメージ1.4倍",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_multiply", target: "enemy_single", value: 1.4 }],
    probability: 25,
  },
  power_smash: {
    id: "power_smash",
    name: "剛撃",
    description: "攻撃時、+80ダメージ（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_add", target: "enemy_single", value: 80 }],
  },
  flash_cut: {
    id: "flash_cut",
    name: "閃光斬",
    description: "攻撃時、40%でダメージ1.2倍",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_multiply", target: "enemy_single", value: 1.2 }],
    probability: 40,
  },
  heavy_impact: {
    id: "heavy_impact",
    name: "重衝撃",
    description: "攻撃時、ダメージ1.1倍",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_multiply", target: "enemy_single", value: 1.1 }],
  },
  sharp_thrust: {
    id: "sharp_thrust",
    name: "鋭突",
    description: "攻撃時、+50ダメージ（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_add", target: "enemy_single", value: 50 }],
  },
  swift_blade: {
    id: "swift_blade",
    name: "迅刃",
    description: "攻撃時、SPEED×0.5を追加ダメージ",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "damage_add", target: "enemy_single", value: 0.5 }],
  },

  // === 複数攻撃・追加攻撃系 ===
  twin_strike: {
    id: "twin_strike",
    name: "双連撃",
    description: "攻撃時、20%で追加攻撃",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "extra_attack", target: "enemy_single", value: 1 }],
    probability: 20,
  },
  triple_slash: {
    id: "triple_slash",
    name: "三連斬",
    description: "攻撃時、10%で2回追加攻撃",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "extra_attack", target: "enemy_single", value: 2 }],
    probability: 10,
  },
  whirlwind: {
    id: "whirlwind",
    name: "旋風撃",
    description: "攻撃時、敵全体に40ダメージ+現在HP3%（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [
      { type: "true_damage", target: "enemy_all", value: 40 },
      { type: "percent_damage", target: "enemy_all", value: 0.03 },
    ],
  },

  // === 吸収・回復系 ===
  life_drain: {
    id: "life_drain",
    name: "生命吸収",
    description: "攻撃時、与ダメージの25%を回復",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "absorb", target: "self", value: 0.25 }],
  },
  monster_steal: {
    id: "monster_steal",
    name: "奪命の一撃",
    description: "攻撃時、敵から30魔獣数を奪う（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "monster_steal", target: "enemy_single", value: 30 }],
  },
  heal_strike: {
    id: "heal_strike",
    name: "癒しの剣",
    description: "攻撃時、HP最低の味方を30回復（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "heal", target: "ally_lowest_hp", value: 30 }],
  },

  // === 防御無視・特殊ダメージ系 ===
  armor_break: {
    id: "armor_break",
    name: "破甲撃",
    description: "攻撃時、敵の防御を無視",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "true_damage", target: "enemy_single", value: 0 }],
  },
  percent_cut: {
    id: "percent_cut",
    name: "割合斬り",
    description: "攻撃時、敵の現在HPの20%ダメージ（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "percent_damage", target: "enemy_single", value: 0.2 }],
  },
  execute_blade: {
    id: "execute_blade",
    name: "処刑剣",
    description: "攻撃時、敵HP30%以下で即死",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "execute", target: "enemy_single", value: 0.3 }],
  },

  // === 状態異常付与系 ===
  blaze_edge: {
    id: "blaze_edge",
    name: "炎刃",
    description: "攻撃時、敵に炎上付与（3ターン、毎ターン30ダメージ、スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "burn", target: "enemy_single", value: 30, duration: { turns: 3 } }],
  },
  venom_fang: {
    id: "venom_fang",
    name: "毒牙",
    description: "攻撃時、敵に毒付与（3ターン、毎ターン20ダメージ、スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "poison", target: "enemy_single", value: 20, duration: { turns: 3 } }],
  },
  frost_blade: {
    id: "frost_blade",
    name: "氷刃",
    description: "攻撃時、25%で敵を凍結（1ターン行動不能）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "freeze", target: "enemy_single", value: 1, duration: { turns: 1 } }],
    probability: 25,
  },
  thunder_strike: {
    id: "thunder_strike",
    name: "雷撃",
    description: "攻撃時、30%で敵を気絶（1ターン行動不能）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "stun", target: "enemy_single", value: 1, duration: { turns: 1 } }],
    probability: 30,
  },
  curse_touch: {
    id: "curse_touch",
    name: "呪縛の手",
    description: "攻撃時、敵を脆弱化（被ダメージ20%増加、2ターン）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "vulnerable", target: "enemy_single", value: 0.2, duration: { turns: 2 } }],
  },
  silence_cut: {
    id: "silence_cut",
    name: "封魔斬",
    description: "攻撃時、40%で敵を沈黙（スキル使用不可、2ターン）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "silence", target: "enemy_single", value: 1, duration: { turns: 2 } }],
    probability: 40,
  },

  // === バフ付与系 ===
  battle_cry: {
    id: "battle_cry",
    name: "鼓舞",
    description: "攻撃時、味方全員の攻撃力+10%（2ターン）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "attack_buff", target: "ally_unit", value: 0.1, duration: { turns: 2 } }],
  },
  shield_bash: {
    id: "shield_bash",
    name: "盾撃",
    description: "攻撃時、自分にシールド30付与（スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "shield", target: "self", value: 30 }],
  },

  // === デバフ解除・バフ解除系 ===
  purify_strike: {
    id: "purify_strike",
    name: "浄化の一撃",
    description: "攻撃時、自分のデバフを1つ解除",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "cleanse", target: "self", value: 1 }],
  },
  dispel_blow: {
    id: "dispel_blow",
    name: "破魔撃",
    description: "攻撃時、敵のバフを1つ解除",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "dispel", target: "enemy_single", value: 1 }],
  },

  // === 特殊系 ===
  mark_target: {
    id: "mark_target",
    name: "狙撃",
    description: "攻撃時、敵にマーク付与（被ダメージ+50、3ターン、スキルLvで強化）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "mark", target: "enemy_single", value: 50, duration: { turns: 3 } }],
  },
  taunt_blow: {
    id: "taunt_blow",
    name: "挑発撃",
    description: "攻撃時、自分に挑発付与（敵の攻撃を引きつける）",
    category: "active",
    timing: "on_attack",
    effects: [{ type: "taunt", target: "self", value: 1, duration: { turns: 2 } }],
  },
};

/** ユニークスキル一覧（特別キャラ専用） */
export const UNIQUE_SKILLS: Record<string, Skill> = {
  // === 攻撃強化系 ===
  divine_spear: {
    id: "divine_spear",
    name: "神槍・絶命",
    description: "攻撃時、ダメージ1.6倍",
    category: "unique",
    timing: "on_attack",
    effects: [{ type: "damage_multiply", target: "enemy_single", value: 1.6 }],
  },
  thunder_call: {
    id: "thunder_call",
    name: "雷神招来",
    description: "攻撃時、50%で敵全体に+100ダメージ（スキルLvで強化）",
    category: "unique",
    timing: "on_attack",
    effects: [{ type: "true_damage", target: "enemy_all", value: 100 }],
    probability: 50,
  },

  // === 蘇生・不死系 ===
  fake_death: {
    id: "fake_death",
    name: "偽りの死",
    description: "死亡時、50%で魔獣数半分で復活",
    category: "unique",
    timing: "on_death",
    effects: [{ type: "revive", target: "self", value: 0.5 }],
    probability: 50,
  },
  phoenix_rebirth: {
    id: "phoenix_rebirth",
    name: "不死鳥の転生",
    description: "死亡時、100%で魔獣数全快で復活（1回のみ）",
    category: "unique",
    timing: "on_death",
    effects: [{ type: "revive", target: "self", value: 1.0 }],
  },

  // === バフ系 ===
  beauty_charm: {
    id: "beauty_charm",
    name: "美神の輝き",
    description: "戦闘開始時、味方全員の魔獣数1.5倍",
    category: "unique",
    timing: "battle_start",
    effects: [{ type: "monster_multiply", target: "ally_unit", value: 1.5 }],
  },
  war_god_blessing: {
    id: "war_god_blessing",
    name: "軍神の加護",
    description: "戦闘開始時、味方全員に無敵（1回攻撃無効）",
    category: "unique",
    timing: "battle_start",
    effects: [{ type: "invincible", target: "ally_unit", value: 1 }],
  },

  // === 即死・処刑系 ===
  death_sentence: {
    id: "death_sentence",
    name: "死刑宣告",
    description: "攻撃時、敵HP50%以下で即死",
    category: "unique",
    timing: "on_attack",
    effects: [{ type: "execute", target: "enemy_single", value: 0.5 }],
  },
  soul_reap: {
    id: "soul_reap",
    name: "魂狩り",
    description: "敵を倒した時、その敵の魔獣数の50%を獲得",
    category: "unique",
    timing: "on_kill",
    effects: [{ type: "monster_steal", target: "enemy_single", value: 0.5 }],
  },

  // === 状態異常系 ===
  absolute_zero: {
    id: "absolute_zero",
    name: "絶対零度",
    description: "戦闘開始時、敵全員を凍結（1ターン行動不能）",
    category: "unique",
    timing: "battle_start",
    effects: [{ type: "freeze", target: "enemy_all", value: 1, duration: { turns: 1 } }],
  },
  plague_touch: {
    id: "plague_touch",
    name: "疫病の手",
    description: "攻撃時、敵全員に毒付与（毎ターン50ダメージ、3ターン、スキルLvで強化）",
    category: "unique",
    timing: "on_attack",
    effects: [{ type: "poison", target: "enemy_all", value: 50, duration: { turns: 3 } }],
  },

  // === 特殊系 ===
  time_stop: {
    id: "time_stop",
    name: "時間停止",
    description: "戦闘開始時、味方全員が2回行動",
    category: "unique",
    timing: "battle_start",
    effects: [{ type: "extra_attack", target: "ally_unit", value: 1 }],
  },
  mirror_force: {
    id: "mirror_force",
    name: "鏡像反転",
    description: "戦闘開始時、受けたダメージを100%反射（1回）",
    category: "unique",
    timing: "battle_start",
    effects: [{ type: "damage_reflect", target: "self", value: 1.0 }],
  },
  dimensional_rift: {
    id: "dimensional_rift",
    name: "次元断裂",
    description: "攻撃時、敵のバフを全て解除し、自分にコピー",
    category: "unique",
    timing: "on_attack",
    effects: [
      { type: "dispel", target: "enemy_single", value: 99 },
      { type: "copy_buff", target: "self", value: 1 },
    ],
  },
  sacrifice: {
    id: "sacrifice",
    name: "生贄の儀",
    description: "自分のHP半分を消費し、味方全員の魔獣数2倍",
    category: "unique",
    timing: "battle_start",
    effects: [
      { type: "percent_damage", target: "self", value: 0.5 },
      { type: "monster_multiply", target: "ally_unit", value: 2.0 },
    ],
  },
};

// =============================================================================
// キャラクター別スキル割り当て
// =============================================================================

/** キャラクターインデックス → スキルセット */
export const CHARACTER_SKILLS: Record<number, CharacterSkills> = {
  // オーディン（インデックス0）: パッシブ + アクティブ + ユニーク
  0: {
    passive: PASSIVE_SKILLS.power_aura,
    active: ACTIVE_SKILLS.critical_edge,
    unique: UNIQUE_SKILLS.divine_spear,
  },
  // トール（インデックス1）: パッシブ + アクティブ + ユニーク
  1: {
    passive: PASSIVE_SKILLS.rage_aura,
    active: ACTIVE_SKILLS.power_smash,
    unique: UNIQUE_SKILLS.thunder_call,
  },
  // ロキ（インデックス2）: アクティブ + ユニーク
  2: {
    active: ACTIVE_SKILLS.twin_strike,
    unique: UNIQUE_SKILLS.fake_death,
  },
  // フレイヤ（インデックス3）: パッシブ + アクティブ + ユニーク
  3: {
    passive: PASSIVE_SKILLS.life_blessing,
    active: ACTIVE_SKILLS.life_drain,
    unique: UNIQUE_SKILLS.beauty_charm,
  },
  // フレイ（インデックス4）: パッシブ + アクティブ
  4: {
    passive: PASSIVE_SKILLS.wind_blessing,
    active: ACTIVE_SKILLS.swift_blade,
  },
  // ヘイムダル（インデックス5）: パッシブ + アクティブ
  5: {
    passive: PASSIVE_SKILLS.steel_guard,
    active: ACTIVE_SKILLS.sharp_thrust,
  },
  // バルドル（インデックス6）: アクティブのみ
  6: {
    active: ACTIVE_SKILLS.flash_cut,
  },
  // ティール（インデックス7）: パッシブ + アクティブ
  7: {
    passive: PASSIVE_SKILLS.power_aura,
    active: ACTIVE_SKILLS.heavy_impact,
  },
  // ニョルド（インデックス8）: アクティブのみ
  8: {
    active: ACTIVE_SKILLS.blaze_edge,
  },
  // ウール（インデックス9）: アクティブのみ
  9: {
    active: ACTIVE_SKILLS.armor_break,
  },
};

