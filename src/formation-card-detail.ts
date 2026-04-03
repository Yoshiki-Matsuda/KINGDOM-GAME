import { getBodyDisplayName, getCharacterIllustrationPath, getCharacterStats } from "./game/characters";
import { getCharacterSkills } from "./game/skills";
import { escapeHtml } from "./utils";

export function buildFormationCardDetailHtml(charIndex: number): string {
  const stats = getCharacterStats(charIndex);
  const skills = getCharacterSkills(charIndex);
  const skillLines: string[] = [];

  if (skills.passive) {
    skillLines.push(`[P] ${skills.passive.name}: ${skills.passive.description}`);
  }
  skillLines.push(`[A] ${skills.active.name}: ${skills.active.description}`);
  if (skills.unique) {
    skillLines.push(`[U] ${skills.unique.name}: ${skills.unique.description}`);
  }

  return `
    <div class="formation-card-detail-header">
      <img src="${getCharacterIllustrationPath(charIndex)}" alt="${escapeHtml(getBodyDisplayName(charIndex))}" class="formation-card-detail-img" />
      <div class="formation-card-detail-name">${escapeHtml(getBodyDisplayName(charIndex))}</div>
    </div>
    <div class="formation-card-detail-stats">
      <div>魔獣数: ${stats.monster_count}</div>
      <div>SPEED: ${stats.speed} / 射程: ${stats.range}</div>
      <div>攻撃: ${stats.attack} / 知力: ${stats.intelligence}</div>
      <div>防御: ${stats.defense} / 魔防: ${stats.magicDefense}</div>
    </div>
    <div class="formation-card-detail-skills">
      <div class="formation-card-detail-skills-title">スキル</div>
      ${skillLines.map((line) => `<div class="formation-card-detail-skill">${escapeHtml(line)}</div>`).join("")}
    </div>
  `;
}
