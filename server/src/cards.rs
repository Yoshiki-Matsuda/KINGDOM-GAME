//! カード定義（プレイヤー・敵共通）

use serde::{Deserialize, Serialize};

/// カードのレアリティ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// カードのステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardStats {
    pub energy: u32,
    pub speed: u32,
    pub attack: u32,
    pub magic: u32,
    pub defense: u32,
    pub magic_defense: u32,
}

impl Default for CardStats {
    fn default() -> Self {
        Self {
            energy: 10,
            speed: 5,
            attack: 5,
            magic: 5,
            defense: 3,
            magic_defense: 3,
        }
    }
}

/// カード定義
#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: u32,
    pub name: &'static str,
    pub rarity: CardRarity,
    pub stats: CardStats,
}

/// 全カード定義
pub const CARDS: &[CardDef] = &[
    // === プレイヤー初期カード（北欧神話） ===
    CardDef { id: 0, name: "オーディン", rarity: CardRarity::Rare, stats: CardStats { energy: 15, speed: 4, attack: 8, magic: 3, defense: 5, magic_defense: 3 } },
    CardDef { id: 1, name: "トール", rarity: CardRarity::Rare, stats: CardStats { energy: 18, speed: 3, attack: 12, magic: 2, defense: 6, magic_defense: 2 } },
    CardDef { id: 2, name: "ロキ", rarity: CardRarity::Rare, stats: CardStats { energy: 10, speed: 7, attack: 4, magic: 10, defense: 2, magic_defense: 5 } },
    CardDef { id: 3, name: "フレイヤ", rarity: CardRarity::Rare, stats: CardStats { energy: 12, speed: 5, attack: 3, magic: 9, defense: 3, magic_defense: 7 } },
    CardDef { id: 4, name: "フレイ", rarity: CardRarity::Rare, stats: CardStats { energy: 14, speed: 5, attack: 7, magic: 6, defense: 4, magic_defense: 4 } },
    CardDef { id: 5, name: "ヘイムダル", rarity: CardRarity::Rare, stats: CardStats { energy: 16, speed: 4, attack: 5, magic: 3, defense: 8, magic_defense: 6 } },
    CardDef { id: 6, name: "バルドル", rarity: CardRarity::Rare, stats: CardStats { energy: 13, speed: 5, attack: 6, magic: 7, defense: 4, magic_defense: 5 } },
    CardDef { id: 7, name: "ティール", rarity: CardRarity::Rare, stats: CardStats { energy: 14, speed: 6, attack: 9, magic: 2, defense: 5, magic_defense: 2 } },
    CardDef { id: 8, name: "ニョルド", rarity: CardRarity::Rare, stats: CardStats { energy: 12, speed: 5, attack: 4, magic: 8, defense: 3, magic_defense: 6 } },
    CardDef { id: 9, name: "ウール", rarity: CardRarity::Rare, stats: CardStats { energy: 11, speed: 8, attack: 7, magic: 4, defense: 3, magic_defense: 3 } },

    // === フィールド敵（Lv1〜6） ===
    CardDef { id: 10, name: "スライム", rarity: CardRarity::Common, stats: CardStats { energy: 2, speed: 3, attack: 3, magic: 2, defense: 2, magic_defense: 1 } },
    CardDef { id: 11, name: "ゴブリン", rarity: CardRarity::Common, stats: CardStats { energy: 4, speed: 4, attack: 5, magic: 3, defense: 3, magic_defense: 2 } },
    CardDef { id: 12, name: "オーク", rarity: CardRarity::Uncommon, stats: CardStats { energy: 6, speed: 3, attack: 8, magic: 4, defense: 6, magic_defense: 3 } },
    CardDef { id: 13, name: "骸骨戦士", rarity: CardRarity::Uncommon, stats: CardStats { energy: 8, speed: 4, attack: 10, magic: 6, defense: 8, magic_defense: 5 } },
    CardDef { id: 14, name: "オーガ", rarity: CardRarity::Rare, stats: CardStats { energy: 12, speed: 3, attack: 15, magic: 8, defense: 12, magic_defense: 6 } },
    CardDef { id: 15, name: "ワイバーン", rarity: CardRarity::Epic, stats: CardStats { energy: 15, speed: 6, attack: 20, magic: 15, defense: 15, magic_defense: 10 } },

    // === 遺跡敵（ノーマル） ===
    CardDef { id: 20, name: "ゴーレム", rarity: CardRarity::Uncommon, stats: CardStats { energy: 8, speed: 2, attack: 6, magic: 2, defense: 10, magic_defense: 4 } },
    CardDef { id: 21, name: "ファントム", rarity: CardRarity::Uncommon, stats: CardStats { energy: 6, speed: 5, attack: 4, magic: 8, defense: 3, magic_defense: 8 } },
    CardDef { id: 22, name: "スケルトンナイト", rarity: CardRarity::Uncommon, stats: CardStats { energy: 10, speed: 4, attack: 10, magic: 3, defense: 8, magic_defense: 4 } },
    CardDef { id: 23, name: "スライムキング", rarity: CardRarity::Rare, stats: CardStats { energy: 12, speed: 4, attack: 5, magic: 12, defense: 4, magic_defense: 6 } },
    CardDef { id: 24, name: "トレジャーミミック", rarity: CardRarity::Rare, stats: CardStats { energy: 7, speed: 7, attack: 8, magic: 5, defense: 5, magic_defense: 5 } },
    CardDef { id: 25, name: "毒蜘蛛", rarity: CardRarity::Uncommon, stats: CardStats { energy: 5, speed: 6, attack: 6, magic: 4, defense: 3, magic_defense: 3 } },

    // === 遺跡敵（レア） ===
    CardDef { id: 30, name: "ダークウィザード", rarity: CardRarity::Rare, stats: CardStats { energy: 12, speed: 5, attack: 12, magic: 10, defense: 6, magic_defense: 8 } },
    CardDef { id: 31, name: "呪われた鎧", rarity: CardRarity::Rare, stats: CardStats { energy: 14, speed: 3, attack: 14, magic: 4, defense: 12, magic_defense: 6 } },
    CardDef { id: 32, name: "シャドウアサシン", rarity: CardRarity::Rare, stats: CardStats { energy: 10, speed: 8, attack: 16, magic: 2, defense: 6, magic_defense: 4 } },
    CardDef { id: 33, name: "炎の精霊", rarity: CardRarity::Rare, stats: CardStats { energy: 9, speed: 5, attack: 6, magic: 14, defense: 4, magic_defense: 10 } },
    CardDef { id: 34, name: "氷の精霊", rarity: CardRarity::Rare, stats: CardStats { energy: 9, speed: 5, attack: 6, magic: 14, defense: 4, magic_defense: 10 } },
    CardDef { id: 35, name: "デスナイト", rarity: CardRarity::Epic, stats: CardStats { energy: 16, speed: 4, attack: 12, magic: 6, defense: 14, magic_defense: 8 } },
    CardDef { id: 36, name: "ネクロマンサー", rarity: CardRarity::Epic, stats: CardStats { energy: 11, speed: 5, attack: 8, magic: 16, defense: 5, magic_defense: 12 } },
    CardDef { id: 37, name: "クリスタルゴーレム", rarity: CardRarity::Epic, stats: CardStats { energy: 14, speed: 4, attack: 10, magic: 12, defense: 12, magic_defense: 12 } },

    // === 遺跡敵（レジェンダリー） ===
    CardDef { id: 40, name: "遺跡の守護者", rarity: CardRarity::Legendary, stats: CardStats { energy: 25, speed: 3, attack: 18, magic: 10, defense: 20, magic_defense: 15 } },
    CardDef { id: 41, name: "ドラゴンゾンビ", rarity: CardRarity::Legendary, stats: CardStats { energy: 30, speed: 4, attack: 25, magic: 15, defense: 18, magic_defense: 12 } },
    CardDef { id: 42, name: "リッチロード", rarity: CardRarity::Legendary, stats: CardStats { energy: 22, speed: 5, attack: 15, magic: 25, defense: 12, magic_defense: 20 } },
    CardDef { id: 43, name: "タイタンコロッサス", rarity: CardRarity::Legendary, stats: CardStats { energy: 35, speed: 2, attack: 22, magic: 12, defense: 25, magic_defense: 18 } },
];

/// カードIDからカード定義を取得
pub fn get_card(id: u32) -> Option<&'static CardDef> {
    CARDS.iter().find(|c| c.id == id)
}

fn normalize_card_name(name: &str) -> &str {
    match name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C') {
        "カースドアーマー" => "呪われた鎧",
        "フレイムスピリット" => "炎の精霊",
        "アイスエレメンタル" => "氷の精霊",
        "ポイズンスパイダー" => "毒蜘蛛",
        normalized => normalized,
    }
}

/// カード名からカード定義を取得
pub fn get_card_by_name(name: &str) -> Option<&'static CardDef> {
    let normalized = normalize_card_name(name);
    CARDS.iter().find(|c| c.name == normalized)
}

/// カード名からカードIDを取得
pub fn get_card_id_by_name(name: &str) -> Option<u32> {
    get_card_by_name(name).map(|c| c.id)
}

/// レアリティに応じたドロップ確率を取得
pub fn get_card_drop_chance(rarity: CardRarity) -> f32 {
    match rarity {
        CardRarity::Common => 0.15,      // 15%
        CardRarity::Uncommon => 0.08,    // 8%
        CardRarity::Rare => 0.03,        // 3%
        CardRarity::Epic => 0.01,        // 1%
        CardRarity::Legendary => 0.002,  // 0.2%
    }
}
