/**
 * スキルシステムの型定義
 */

/** スキルの発動タイミング */
export type SkillTiming =
  | "battle_start"      // 戦闘開始時（パッシブ）
  | "on_attack"         // 攻撃時（アクティブ）
  | "on_defend"         // 防御時
  | "on_damage"         // ダメージを受けた時
  | "on_kill"           // 敵を倒した時
  | "on_death"          // 自分が倒れた時
  | "turn_start"        // ターン開始時
  | "turn_end"          // ターン終了時
  | "hp_low"            // HP低下時（50%以下など）
  | "first_attack"      // 初回攻撃時
  | "continuous";       // 常時発動

/** スキルの効果対象 */
export type SkillTarget =
  | "self"              // 自分のみ
  | "ally_unit"         // ユニット内の味方全員
  | "ally_single"       // ユニット内の味方1体（ランダム）
  | "ally_lowest_hp"    // HP最低の味方
  | "enemy_single"      // 敵1体（現在の対象）
  | "enemy_all"         // 敵全体
  | "enemy_random"      // ランダムな敵1体
  | "enemy_highest_hp"  // HP最大の敵
  | "both_all";         // 敵味方全員

/** スキルの効果タイプ */
export type SkillEffectType =
  // === ステータス変更系 ===
  | "monster_multiply"   // 魔獣数倍率
  | "monster_add"        // 魔獣数加算
  | "monster_set"        // 魔獣数固定値設定
  | "speed_multiply"    // SPEED倍率
  | "speed_add"         // SPEED加算
  | "damage_multiply"   // ダメージ倍率
  | "damage_add"        // ダメージ加算
  | "damage_reduce"     // ダメージ軽減率
  | "damage_reflect"    // ダメージ反射率
  // === 回復・蘇生系 ===
  | "heal"              // 固定値回復
  | "heal_percent"      // 最大HPの%回復
  | "revive"            // 蘇生（valueは復活時HP割合）
  | "absorb"            // 与ダメージ吸収
  // === 攻撃系 ===
  | "extra_attack"      // 追加攻撃回数
  | "true_damage"       // 固定ダメージ（防御無視）
  | "percent_damage"    // 現在HP割合ダメージ
  | "execute"           // 処刑（HP一定以下で即死）
  // === 防御系 ===
  | "shield"            // シールド付与（ダメージ吸収）
  | "invincible"        // 無敵（1回攻撃無効）
  | "evasion"           // 回避率上昇
  | "counter"           // 反撃
  // === 状態異常系 ===
  | "poison"            // 毒（毎ターンダメージ）
  | "burn"              // 炎上（毎ターンダメージ）
  | "freeze"            // 凍結（行動不能）
  | "stun"              // 気絶（行動不能）
  | "silence"           // 沈黙（スキル使用不可）
  | "blind"             // 暗闘（命中率低下）
  | "weaken"            // 弱体化（ダメージ低下）
  | "vulnerable"        // 脆弱（被ダメージ増加）
  // === バフ・デバフ系 ===
  | "attack_buff"       // 攻撃バフ
  | "defense_buff"      // 防御バフ
  | "speed_buff"        // 速度バフ
  | "critical_buff"     // クリティカル率バフ
  | "attack_debuff"     // 攻撃デバフ
  | "defense_debuff"    // 防御デバフ
  | "cleanse"           // デバフ解除
  | "dispel"            // バフ解除（敵の）
  // === 特殊系 ===
  | "taunt"             // 挑発（自分を攻撃させる）
  | "stealth"           // 隠密（ターゲットされない）
  | "mark"              // マーク（追加ダメージ）
  | "copy_buff"         // バフコピー
  | "transfer_debuff"   // デバフ転送
  | "cooldown_reduce"   // クールダウン短縮
  | "monster_steal"      // 魔獣数奪取
  | "summon";           // 召喚

/** スキル効果の条件 */
export interface SkillCondition {
  type: "hp_below" | "hp_above" | "ally_count" | "enemy_count" | "has_buff" | "has_debuff" | "first_turn" | "random";
  value: number;  // HP%、味方数、確率など
}

/** 状態異常の持続情報 */
export interface StatusEffectDuration {
  turns?: number;      // 持続ターン数
  stacks?: number;     // スタック数
  maxStacks?: number;  // 最大スタック数
}

/** スキル効果の定義 */
export interface SkillEffect {
  type: SkillEffectType;
  target: SkillTarget;
  value: number;  // 効果値（倍率 or 固定値）
  /** 効果発動の追加条件 */
  condition?: SkillCondition;
  /** 状態異常の持続情報 */
  duration?: StatusEffectDuration;
  /** 連鎖効果（この効果発動後に追加で発動する効果） */
  chain?: SkillEffect;
}

/** スキルの種別 */
export type SkillCategory = "passive" | "active" | "unique";

/** スキル定義 */
export interface Skill {
  id: string;
  name: string;
  description: string;
  category: SkillCategory;
  timing: SkillTiming;
  effects: SkillEffect[];
  /** 発動確率（0-100）。未指定は100（確定発動） */
  probability?: number;
}

/** キャラクターのスキルセット */
export interface CharacterSkills {
  passive?: Skill;
  active: Skill;
  unique?: Skill;
}
