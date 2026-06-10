/**
 * スキルシステム — キャラごとのパッシブ・アクティブ・ユニークスキル
 *
 * - パッシブスキル: 戦闘開始時に発動。味方ユニット全体に効果
 * - アクティブスキル: 攻撃時に発動。全キャラが持つ
 * - ユニークスキル: 特別キャラのみ。発動タイミングはスキル自体に定義
 */

import type { CharacterSkills, Skill } from "./types";
import { ACTIVE_SKILLS, CHARACTER_SKILLS } from "./data";

export * from "./types";
export * from "./data";

/** デフォルトのアクティブスキル（スキル未設定キャラ用） */
export const DEFAULT_ACTIVE_SKILL: Skill = ACTIVE_SKILLS.sharp_thrust;

/** キャラのスキルセットを取得（未定義キャラはデフォルトスキルを返す） */
export function getCharacterSkills(index: number): CharacterSkills {
  return CHARACTER_SKILLS[index] ?? { active: DEFAULT_ACTIVE_SKILL };
}

/** スキル情報をサーバーに送信する形式に変換 */
export interface SkillData {
  passive_id?: string;
  active_id: string;
  unique_id?: string;
}

export function getCharacterSkillData(index: number): SkillData {
  const skills = getCharacterSkills(index);
  return {
    passive_id: skills.passive?.id,
    active_id: skills.active.id,
    unique_id: skills.unique?.id,
  };
}

/** ユニット全体のスキルデータを取得 */
export function getUnitSkillData(indices: number[]): SkillData[] {
  return indices.map(getCharacterSkillData);
}
