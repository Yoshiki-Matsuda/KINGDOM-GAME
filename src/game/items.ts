/**
 * アイテム定義とインベントリ管理
 */

/** アイテムのレアリティ */
export type ItemRarity = "common" | "uncommon" | "rare" | "epic" | "legendary";

/** アイテムのカテゴリ */
export type ItemCategory = "material" | "skill_book" | "special";

/** アイテム定義 */
export interface ItemDef {
  id: string;
  name: string;
  description: string;
  rarity: ItemRarity;
  category: ItemCategory;
  icon: string;
  maxStack: number;
}

/** 全アイテム定義 */
export const ITEMS: Record<string, ItemDef> = {
  // === 基本素材（Common） ===
  ancient_stone: {
    id: "ancient_stone",
    name: "古代の石材",
    description: "風化した建築用の石。施設建設の基本素材。",
    rarity: "common",
    category: "material",
    icon: "🪨",
    maxStack: 999,
  },
  rusty_gear: {
    id: "rusty_gear",
    name: "錆びた歯車",
    description: "古代機械の部品。工房系施設に必要。",
    rarity: "common",
    category: "material",
    icon: "⚙️",
    maxStack: 999,
  },
  rotten_wood: {
    id: "rotten_wood",
    name: "朽ちた木材",
    description: "長年放置された木材。加工すれば使える。",
    rarity: "common",
    category: "material",
    icon: "🪵",
    maxStack: 999,
  },
  broken_brick: {
    id: "broken_brick",
    name: "砕けたレンガ",
    description: "崩れた壁の残骸。建築素材になる。",
    rarity: "common",
    category: "material",
    icon: "🧱",
    maxStack: 999,
  },

  // === 中級素材（Uncommon） ===
  mystic_crystal: {
    id: "mystic_crystal",
    name: "神秘の水晶",
    description: "魔力を帯びた結晶。魔法施設に必須。",
    rarity: "uncommon",
    category: "material",
    icon: "💎",
    maxStack: 999,
  },
  magic_shard: {
    id: "magic_shard",
    name: "魔力の欠片",
    description: "凝縮された魔力。研究や強化に使用。",
    rarity: "uncommon",
    category: "material",
    icon: "✨",
    maxStack: 999,
  },
  refined_iron: {
    id: "refined_iron",
    name: "精錬された鉄",
    description: "高品質な金属。武具や施設に使用。",
    rarity: "uncommon",
    category: "material",
    icon: "🔩",
    maxStack: 999,
  },
  reinforced_fiber: {
    id: "reinforced_fiber",
    name: "強化繊維",
    description: "丈夫な布素材。様々な用途に使える。",
    rarity: "uncommon",
    category: "material",
    icon: "🧵",
    maxStack: 999,
  },
  ancient_blueprint: {
    id: "ancient_blueprint",
    name: "古代の設計図",
    description: "失われた技術の断片。上級施設に必要。",
    rarity: "uncommon",
    category: "material",
    icon: "📜",
    maxStack: 999,
  },

  // === 高級素材（Rare） ===
  shining_magicstone: {
    id: "shining_magicstone",
    name: "輝く魔石",
    description: "強力な魔力を秘めた石。貴重な素材。",
    rarity: "rare",
    category: "material",
    icon: "💠",
    maxStack: 999,
  },
  golden_gear: {
    id: "golden_gear",
    name: "黄金の歯車",
    description: "伝説の機械部品。最高級の施設に必要。",
    rarity: "rare",
    category: "material",
    icon: "🔆",
    maxStack: 999,
  },

  // === 最高級素材（Epic/Legendary） ===
  guardian_core: {
    id: "guardian_core",
    name: "守護者の核",
    description: "遺跡の守護者から得られる核。強大な力を秘める。",
    rarity: "epic",
    category: "material",
    icon: "🔮",
    maxStack: 99,
  },
  ancient_kings_seal: {
    id: "ancient_kings_seal",
    name: "古代王の印章",
    description: "古代王国の証。最高級施設の建設に必要。",
    rarity: "epic",
    category: "material",
    icon: "👑",
    maxStack: 99,
  },
  dragon_scale: {
    id: "dragon_scale",
    name: "ドラゴンの鱗",
    description: "伝説のドラゴンから得た鱗。究極の素材。",
    rarity: "legendary",
    category: "material",
    icon: "🐉",
    maxStack: 50,
  },

  // === スキルの書 ===
  skill_book_attack: {
    id: "skill_book_attack",
    name: "スキルの書【攻撃】",
    description: "攻撃系スキルを習得できる書物。",
    rarity: "rare",
    category: "skill_book",
    icon: "📕",
    maxStack: 10,
  },
  skill_book_defense: {
    id: "skill_book_defense",
    name: "スキルの書【防御】",
    description: "防御系スキルを習得できる書物。",
    rarity: "rare",
    category: "skill_book",
    icon: "📗",
    maxStack: 10,
  },
  skill_book_support: {
    id: "skill_book_support",
    name: "スキルの書【支援】",
    description: "支援系スキルを習得できる書物。",
    rarity: "rare",
    category: "skill_book",
    icon: "📘",
    maxStack: 10,
  },

  // === 特殊アイテム ===
  exp_crystal: {
    id: "exp_crystal",
    name: "経験値の結晶",
    description: "カードの成長に使用できる結晶。",
    rarity: "uncommon",
    category: "special",
    icon: "⭐",
    maxStack: 999,
  },
  summon_shard: {
    id: "summon_shard",
    name: "召喚の欠片",
    description: "集めるとカードを召喚できる。",
    rarity: "rare",
    category: "special",
    icon: "🌟",
    maxStack: 999,
  },

  // === 通貨・ショップ用 ===
  gold: {
    id: "gold",
    name: "ゴールド",
    description: "基本通貨。様々なものを購入できる。",
    rarity: "common",
    category: "special",
    icon: "🪙",
    maxStack: 999999,
  },
  card_pack_ticket: {
    id: "card_pack_ticket",
    name: "カードパックチケット",
    description: "カードショップでノーマルパックを1つ開封できる。",
    rarity: "uncommon",
    category: "special",
    icon: "🎫",
    maxStack: 999,
  },
  rare_pack_ticket: {
    id: "rare_pack_ticket",
    name: "レアパックチケット",
    description: "カードショップでレアパックを1つ開封できる。レア以上確定。",
    rarity: "rare",
    category: "special",
    icon: "🎟️",
    maxStack: 99,
  },
};

/** アイテムIDからアイテム定義を取得 */
export function getItem(id: string): ItemDef | undefined {
  return ITEMS[id];
}

/** レアリティに応じた色を取得 */
export function getRarityColor(rarity: ItemRarity): string {
  switch (rarity) {
    case "common": return "#9ca3af";
    case "uncommon": return "#22c55e";
    case "rare": return "#3b82f6";
    case "epic": return "#a855f7";
    case "legendary": return "#f59e0b";
  }
}

import type { InventoryItem } from "../shared/game-state";

export type { InventoryItem };

/** インベントリ操作 */
export function addItemToInventory(
  inventory: InventoryItem[],
  itemId: string,
  count: number
): InventoryItem[] {
  const existing = inventory.find(i => i.item_id === itemId);
  if (existing) {
    existing.count += count;
    const maxStack = ITEMS[itemId]?.maxStack ?? 999;
    existing.count = Math.min(existing.count, maxStack);
  } else {
    inventory.push({ item_id: itemId, count });
  }
  return inventory;
}

export function removeItemFromInventory(
  inventory: InventoryItem[],
  itemId: string,
  count: number
): boolean {
  const existing = inventory.find(i => i.item_id === itemId);
  if (!existing || existing.count < count) return false;
  existing.count -= count;
  return true;
}

export function getItemCount(inventory: InventoryItem[], itemId: string): number {
  return inventory.find(i => i.item_id === itemId)?.count ?? 0;
}
