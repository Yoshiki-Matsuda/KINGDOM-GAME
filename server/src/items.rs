//! アイテム定義とドロップシステム

use rand::Rng;
use serde::{Deserialize, Serialize};

/// インベントリ内のアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub count: u32,
}

/// ドロップアイテム（確率付き）
#[derive(Debug, Clone)]
pub struct DropEntry {
    pub item_id: &'static str,
    pub min_count: u32,
    pub max_count: u32,
    pub chance: f32, // 0.0 - 1.0
}

/// 全アイテムID定数
pub mod item_ids {
    // 基本素材
    pub const ANCIENT_STONE: &str = "ancient_stone";
    pub const RUSTY_GEAR: &str = "rusty_gear";
    pub const ROTTEN_WOOD: &str = "rotten_wood";
    pub const BROKEN_BRICK: &str = "broken_brick";
    
    // 中級素材
    pub const MYSTIC_CRYSTAL: &str = "mystic_crystal";
    pub const MAGIC_SHARD: &str = "magic_shard";
    pub const REFINED_IRON: &str = "refined_iron";
    pub const REINFORCED_FIBER: &str = "reinforced_fiber";
    pub const ANCIENT_BLUEPRINT: &str = "ancient_blueprint";
    
    // 高級素材
    pub const SHINING_MAGICSTONE: &str = "shining_magicstone";
    pub const GOLDEN_GEAR: &str = "golden_gear";
    
    // 最高級素材
    pub const GUARDIAN_CORE: &str = "guardian_core";
    pub const ANCIENT_KINGS_SEAL: &str = "ancient_kings_seal";
    pub const DRAGON_SCALE: &str = "dragon_scale";
    
    // スキルの書
    pub const SKILL_BOOK_ATTACK: &str = "skill_book_attack";
    pub const SKILL_BOOK_DEFENSE: &str = "skill_book_defense";
    pub const SKILL_BOOK_SUPPORT: &str = "skill_book_support";
    
    // 特殊
    pub const EXP_CRYSTAL: &str = "exp_crystal";
    pub const SUMMON_SHARD: &str = "summon_shard";
    
    // 通貨・ショップ用
    pub const GOLD: &str = "gold";
    pub const CARD_PACK_TICKET: &str = "card_pack_ticket";
    pub const RARE_PACK_TICKET: &str = "rare_pack_ticket";
}

use item_ids::*;

fn normalize_enemy_type(enemy_type: &str) -> &str {
    match enemy_type.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C') {
        "ゴーレム" => "golem",
        "ファントム" => "phantom",
        "スケルトンナイト" => "skeleton_knight",
        "トレジャーミミック" => "treasure_mimic",
        "ダークウィザード" => "dark_wizard",
        "スライムキング" => "slime_king",
        "カースドアーマー" | "呪われた鎧" => "cursed_armor",
        "シャドウアサシン" => "shadow_assassin",
        "フレイムスピリット" | "炎の精霊" => "flame_spirit",
        "アイスエレメンタル" | "氷の精霊" => "ice_elemental",
        "ポイズンスパイダー" | "毒蜘蛛" => "poison_spider",
        "ストーンガーゴイル" => "stone_gargoyle",
        "デーモンインプ" => "demon_imp",
        "デスナイト" => "death_knight",
        "ネクロマンサー" => "necromancer",
        "クリスタルゴーレム" => "crystal_golem",
        "サンダーホーク" => "thunder_hawk",
        "アースワーム" => "earth_wyrm",
        "ヴォイドストーカー" => "void_stalker",
        "エンシェントマミー" => "ancient_mummy",
        "遺跡の守護者" => "ruin_guardian",
        "ドラゴンゾンビ" => "dragon_zombie",
        "リッチロード" => "lich_lord",
        "タイタンコロッサス" => "titan_colossus",
        normalized => normalized,
    }
}

/// 敵タイプごとのドロップテーブル
pub fn get_drop_table(enemy_type: &str) -> Vec<DropEntry> {
    match normalize_enemy_type(enemy_type) {
        // === 基本敵 ===
        "golem" | "ゴーレム" => vec![
            DropEntry { item_id: GOLD, min_count: 20, max_count: 40, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: BROKEN_BRICK, min_count: 1, max_count: 3, chance: 0.5 },
            DropEntry { item_id: REFINED_IRON, min_count: 1, max_count: 2, chance: 0.2 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.1 },
        ],
        "phantom" | "ファントム" => vec![
            DropEntry { item_id: GOLD, min_count: 15, max_count: 35, chance: 1.0 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 2, max_count: 3, chance: 1.0 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 1, chance: 0.05 },
            DropEntry { item_id: SKILL_BOOK_SUPPORT, min_count: 1, max_count: 1, chance: 0.03 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.08 },
        ],
        "skeleton_knight" | "スケルトンナイト" => vec![
            DropEntry { item_id: GOLD, min_count: 20, max_count: 45, chance: 1.0 },
            DropEntry { item_id: RUSTY_GEAR, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: REINFORCED_FIBER, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: REFINED_IRON, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.1 },
        ],
        "treasure_mimic" | "トレジャーミミック" => vec![
            DropEntry { item_id: GOLD, min_count: 50, max_count: 100, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 2, max_count: 5, chance: 0.6 },
            DropEntry { item_id: RUSTY_GEAR, min_count: 2, max_count: 5, chance: 0.6 },
            DropEntry { item_id: ROTTEN_WOOD, min_count: 2, max_count: 5, chance: 0.6 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 3, chance: 0.5 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 1, max_count: 3, chance: 0.5 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 2, chance: 0.2 },
            DropEntry { item_id: GOLDEN_GEAR, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: SUMMON_SHARD, min_count: 1, max_count: 2, chance: 0.15 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 2, chance: 0.3 },
        ],
        "dark_wizard" | "ダークウィザード" => vec![
            DropEntry { item_id: GOLD, min_count: 30, max_count: 60, chance: 1.0 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.03 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.15 },
        ],
        
        // === 追加敵（低〜中級） ===
        "slime_king" => vec![
            DropEntry { item_id: MAGIC_SHARD, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 2, max_count: 3, chance: 0.5 },
            DropEntry { item_id: SUMMON_SHARD, min_count: 1, max_count: 2, chance: 0.2 },
        ],
        "cursed_armor" => vec![
            DropEntry { item_id: REFINED_IRON, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: RUSTY_GEAR, min_count: 2, max_count: 4, chance: 0.6 },
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: SKILL_BOOK_DEFENSE, min_count: 1, max_count: 1, chance: 0.05 },
        ],
        "shadow_assassin" => vec![
            DropEntry { item_id: REINFORCED_FIBER, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.08 },
        ],
        "flame_spirit" => vec![
            DropEntry { item_id: MAGIC_SHARD, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: EXP_CRYSTAL, min_count: 1, max_count: 2, chance: 0.3 },
        ],
        "ice_elemental" => vec![
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: EXP_CRYSTAL, min_count: 1, max_count: 2, chance: 0.3 },
        ],
        "poison_spider" => vec![
            DropEntry { item_id: REINFORCED_FIBER, min_count: 2, max_count: 3, chance: 1.0 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ROTTEN_WOOD, min_count: 1, max_count: 3, chance: 0.5 },
        ],
        "stone_gargoyle" => vec![
            DropEntry { item_id: ANCIENT_STONE, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: BROKEN_BRICK, min_count: 2, max_count: 4, chance: 0.6 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 2, chance: 0.25 },
        ],
        "demon_imp" => vec![
            DropEntry { item_id: MAGIC_SHARD, min_count: 2, max_count: 3, chance: 1.0 },
            DropEntry { item_id: EXP_CRYSTAL, min_count: 1, max_count: 3, chance: 0.4 },
            DropEntry { item_id: SUMMON_SHARD, min_count: 1, max_count: 1, chance: 0.1 },
        ],
        
        // === 追加敵（中〜上級） ===
        "death_knight" => vec![
            DropEntry { item_id: REFINED_IRON, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: RUSTY_GEAR, min_count: 2, max_count: 4, chance: 0.5 },
            DropEntry { item_id: GOLDEN_GEAR, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.08 },
        ],
        "necromancer" => vec![
            DropEntry { item_id: MAGIC_SHARD, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: SKILL_BOOK_SUPPORT, min_count: 1, max_count: 1, chance: 0.05 },
        ],
        "crystal_golem" => vec![
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 4, max_count: 6, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 2, max_count: 4, chance: 0.5 },
        ],
        "thunder_hawk" => vec![
            DropEntry { item_id: REINFORCED_FIBER, min_count: 2, max_count: 4, chance: 1.0 },
            DropEntry { item_id: MAGIC_SHARD, min_count: 2, max_count: 3, chance: 0.6 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 1, chance: 0.15 },
        ],
        "earth_wyrm" => vec![
            DropEntry { item_id: ANCIENT_STONE, min_count: 4, max_count: 6, chance: 1.0 },
            DropEntry { item_id: REFINED_IRON, min_count: 2, max_count: 4, chance: 0.5 },
            DropEntry { item_id: GUARDIAN_CORE, min_count: 1, max_count: 1, chance: 0.05 },
        ],
        "void_stalker" => vec![
            DropEntry { item_id: MAGIC_SHARD, min_count: 3, max_count: 5, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.1 },
        ],
        "ancient_mummy" => vec![
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 2, chance: 0.8 },
            DropEntry { item_id: REINFORCED_FIBER, min_count: 2, max_count: 4, chance: 0.6 },
            DropEntry { item_id: ANCIENT_KINGS_SEAL, min_count: 1, max_count: 1, chance: 0.05 },
        ],
        
        // === ボス敵 ===
        "ruin_guardian" | "遺跡の守護者" => vec![
            DropEntry { item_id: GOLD, min_count: 100, max_count: 200, chance: 1.0 },
            DropEntry { item_id: GUARDIAN_CORE, min_count: 1, max_count: 1, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 2, max_count: 4, chance: 0.8 },
            DropEntry { item_id: GOLDEN_GEAR, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ANCIENT_KINGS_SEAL, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: DRAGON_SCALE, min_count: 1, max_count: 1, chance: 0.05 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: SKILL_BOOK_DEFENSE, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.2 },
        ],
        "dragon_zombie" | "ドラゴンゾンビ" => vec![
            DropEntry { item_id: GOLD, min_count: 150, max_count: 300, chance: 1.0 },
            DropEntry { item_id: GUARDIAN_CORE, min_count: 1, max_count: 2, chance: 1.0 },
            DropEntry { item_id: DRAGON_SCALE, min_count: 1, max_count: 2, chance: 0.5 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 3, max_count: 5, chance: 0.8 },
            DropEntry { item_id: ANCIENT_KINGS_SEAL, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.25 },
        ],
        "lich_lord" | "リッチロード" => vec![
            DropEntry { item_id: GOLD, min_count: 120, max_count: 250, chance: 1.0 },
            DropEntry { item_id: GUARDIAN_CORE, min_count: 1, max_count: 1, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 3, max_count: 6, chance: 1.0 },
            DropEntry { item_id: ANCIENT_KINGS_SEAL, min_count: 1, max_count: 1, chance: 0.25 },
            DropEntry { item_id: SKILL_BOOK_ATTACK, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: SKILL_BOOK_SUPPORT, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.2 },
        ],
        "titan_colossus" | "タイタンコロッサス" => vec![
            DropEntry { item_id: GOLD, min_count: 200, max_count: 400, chance: 1.0 },
            DropEntry { item_id: GUARDIAN_CORE, min_count: 2, max_count: 3, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 8, max_count: 12, chance: 1.0 },
            DropEntry { item_id: GOLDEN_GEAR, min_count: 2, max_count: 3, chance: 0.6 },
            DropEntry { item_id: ANCIENT_KINGS_SEAL, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: DRAGON_SCALE, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: SKILL_BOOK_DEFENSE, min_count: 1, max_count: 1, chance: 0.2 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 2, chance: 0.3 },
        ],
        
        // === 通常敵（レベル別） ===
        // Lv1: スライム
        "スライム" => vec![
            DropEntry { item_id: GOLD, min_count: 5, max_count: 15, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ROTTEN_WOOD, min_count: 1, max_count: 2, chance: 0.2 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.03 },
        ],
        // Lv2: ゴブリン
        "ゴブリン" => vec![
            DropEntry { item_id: GOLD, min_count: 10, max_count: 25, chance: 1.0 },
            DropEntry { item_id: RUSTY_GEAR, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: ROTTEN_WOOD, min_count: 1, max_count: 3, chance: 0.3 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.05 },
        ],
        // Lv3: オーク
        "オーク" => vec![
            DropEntry { item_id: GOLD, min_count: 20, max_count: 40, chance: 1.0 },
            DropEntry { item_id: REFINED_IRON, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: BROKEN_BRICK, min_count: 1, max_count: 3, chance: 0.3 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.08 },
        ],
        // Lv4: 骸骨戦士
        "骸骨戦士" => vec![
            DropEntry { item_id: GOLD, min_count: 30, max_count: 60, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 2, max_count: 4, chance: 0.5 },
            DropEntry { item_id: ANCIENT_BLUEPRINT, min_count: 1, max_count: 1, chance: 0.1 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.1 },
        ],
        // Lv5: オーガ
        "オーガ" => vec![
            DropEntry { item_id: GOLD, min_count: 50, max_count: 100, chance: 1.0 },
            DropEntry { item_id: REFINED_IRON, min_count: 2, max_count: 4, chance: 0.5 },
            DropEntry { item_id: MYSTIC_CRYSTAL, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.03 },
        ],
        // Lv6: ワイバーン
        "ワイバーン" => vec![
            DropEntry { item_id: GOLD, min_count: 80, max_count: 150, chance: 1.0 },
            DropEntry { item_id: SHINING_MAGICSTONE, min_count: 1, max_count: 2, chance: 0.4 },
            DropEntry { item_id: GOLDEN_GEAR, min_count: 1, max_count: 1, chance: 0.15 },
            DropEntry { item_id: DRAGON_SCALE, min_count: 1, max_count: 1, chance: 0.05 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 2, chance: 0.2 },
            DropEntry { item_id: RARE_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.08 },
        ],
        
        // デフォルト（未定義の敵）
        _ => vec![
            DropEntry { item_id: GOLD, min_count: 10, max_count: 30, chance: 1.0 },
            DropEntry { item_id: ANCIENT_STONE, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: ROTTEN_WOOD, min_count: 1, max_count: 2, chance: 0.3 },
            DropEntry { item_id: CARD_PACK_TICKET, min_count: 1, max_count: 1, chance: 0.05 },
        ],
    }
}

/// ドロップを計算（drop_rate_bonus: ドロップ率ボーナス%、例: 50なら+50%）
pub fn calculate_drops(enemy_types: &[&str], drop_rate_bonus: u32) -> Vec<InventoryItem> {
    let mut rng = rand::thread_rng();
    let mut drops: Vec<InventoryItem> = Vec::new();
    
    // ドロップ率倍率を計算（例: bonus=50 → multiplier=1.5）
    let rate_multiplier = 1.0 + (drop_rate_bonus as f32 / 100.0);
    
    for enemy_type in enemy_types {
        let table = get_drop_table(enemy_type);
        for entry in table {
            // ボーナスを適用した確率（最大100%）
            let boosted_chance = (entry.chance * rate_multiplier).min(1.0);
            if rng.gen::<f32>() <= boosted_chance {
                let count = rng.gen_range(entry.min_count..=entry.max_count);
                add_to_drops(&mut drops, entry.item_id, count);
            }
        }
    }
    
    drops
}

fn add_to_drops(drops: &mut Vec<InventoryItem>, item_id: &str, count: u32) {
    if let Some(existing) = drops.iter_mut().find(|d| d.item_id == item_id) {
        existing.count += count;
    } else {
        drops.push(InventoryItem {
            item_id: item_id.to_string(),
            count,
        });
    }
}

/// インベントリにアイテムを追加
pub fn add_items_to_inventory(inventory: &mut Vec<InventoryItem>, items: Vec<InventoryItem>) {
    for item in items {
        if let Some(existing) = inventory.iter_mut().find(|i| i.item_id == item.item_id) {
            existing.count += item.count;
        } else {
            inventory.push(item);
        }
    }
}

