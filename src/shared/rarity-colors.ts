/** ゲーム内共通のレアリティ段階 */
export type RarityTier = "common" | "uncommon" | "rare" | "epic" | "legendary";

/** レアリティに応じた表示色 */
export function getRarityColor(rarity: RarityTier): string {
  switch (rarity) {
    case "common":
      return "#9ca3af";
    case "uncommon":
      return "#22c55e";
    case "rare":
      return "#3b82f6";
    case "epic":
      return "#a855f7";
    case "legendary":
      return "#f59e0b";
  }
}

/** レアリティ用 CSS クラス名 */
export function getRarityClass(rarity: RarityTier): string {
  return `card-rarity-${rarity}`;
}
