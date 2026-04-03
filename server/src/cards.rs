//! カード定義（プレイヤー・敵共通）

use serde::{Deserialize, Serialize};
use crate::skills::SkillData;

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

/// KC準拠の7種族
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Race {
    Beast,
    Demihuman,
    Demon,
    Dragon,
    Giant,
    Spirit,
    Undead,
}

/// カードのステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardStats {
    pub monster_count: u32,
    pub speed: u32,
    pub attack: u32,
    #[serde(alias = "magic")]
    pub intelligence: u32,
    pub defense: u32,
    pub magic_defense: u32,
    /// 射程 (1=近接, 2=中距離, 3=遠距離)
    #[serde(default = "default_range")]
    pub range: u8,
    /// ユニット編成コスト（0.5刻み。従来KC値の約1/2を四捨五入）
    #[serde(default = "default_card_cost")]
    pub cost: f32,
    /// 占拠力（勝利時に敵拠点耐久を削る）
    #[serde(default = "default_occupation_power")]
    pub occupation_power: u32,
}

fn default_range() -> u8 { 1 }
fn default_card_cost() -> f32 { 1.5 }
fn default_occupation_power() -> u32 {
    100
}

impl Default for CardStats {
    fn default() -> Self {
        Self {
            monster_count: 10,
            speed: 5,
            attack: 5,
            intelligence: 5,
            defense: 3,
            magic_defense: 3,
            range: 1,
            cost: 1.5,
            occupation_power: 100,
        }
    }
}

/// カード定義
#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: u32,
    pub name: &'static str,
    pub rarity: CardRarity,
    pub race: Race,
    pub stats: CardStats,
    pub default_skills: Option<SkillData>,
}

/// 全カード定義
pub const CARDS: &[CardDef] = &[
    // === プレイヤー初期カード（KC 7種族） ===
    CardDef { id: 0, name: "ダイアウルフ", rarity: CardRarity::Rare, race: Race::Beast, stats: CardStats { monster_count: 15, speed: 6, attack: 9, intelligence: 3, defense: 4, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 1, name: "ゴブリンウォリアー", rarity: CardRarity::Rare, race: Race::Demihuman, stats: CardStats { monster_count: 18, speed: 5, attack: 7, intelligence: 4, defense: 6, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 2, name: "インプ", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 10, speed: 7, attack: 4, intelligence: 10, defense: 2, magic_defense: 6, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 3, name: "ワイバーン", rarity: CardRarity::Rare, race: Race::Dragon, stats: CardStats { monster_count: 12, speed: 5, attack: 8, intelligence: 6, defense: 5, magic_defense: 5, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 4, name: "ゴーレム", rarity: CardRarity::Rare, race: Race::Giant, stats: CardStats { monster_count: 16, speed: 3, attack: 6, intelligence: 2, defense: 10, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 5, name: "サラマンダー", rarity: CardRarity::Rare, race: Race::Spirit, stats: CardStats { monster_count: 11, speed: 5, attack: 5, intelligence: 9, defense: 3, magic_defense: 7, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 6, name: "スケルトンソルジャー", rarity: CardRarity::Rare, race: Race::Undead, stats: CardStats { monster_count: 14, speed: 4, attack: 7, intelligence: 3, defense: 7, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 7, name: "ヘルハウンド", rarity: CardRarity::Rare, race: Race::Beast, stats: CardStats { monster_count: 13, speed: 7, attack: 8, intelligence: 4, defense: 3, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 8, name: "リザードマン", rarity: CardRarity::Rare, race: Race::Demihuman, stats: CardStats { monster_count: 14, speed: 5, attack: 8, intelligence: 5, defense: 5, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 9, name: "トレント", rarity: CardRarity::Rare, race: Race::Spirit, stats: CardStats { monster_count: 16, speed: 3, attack: 5, intelligence: 7, defense: 8, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // === フィールド敵（Lv1〜6） ===
    CardDef { id: 10, name: "ゴブリン", rarity: CardRarity::Common, race: Race::Demihuman, stats: CardStats { monster_count: 2, speed: 3, attack: 3, intelligence: 2, defense: 2, magic_defense: 1, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 11, name: "コボルド", rarity: CardRarity::Common, race: Race::Demihuman, stats: CardStats { monster_count: 4, speed: 4, attack: 5, intelligence: 3, defense: 3, magic_defense: 2, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 12, name: "オーク", rarity: CardRarity::Uncommon, race: Race::Demihuman, stats: CardStats { monster_count: 6, speed: 3, attack: 8, intelligence: 4, defense: 6, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 13, name: "スケルトン", rarity: CardRarity::Uncommon, race: Race::Undead, stats: CardStats { monster_count: 8, speed: 4, attack: 10, intelligence: 6, defense: 8, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 14, name: "トロール", rarity: CardRarity::Rare, race: Race::Giant, stats: CardStats { monster_count: 12, speed: 3, attack: 15, intelligence: 3, defense: 12, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 15, name: "ドレイク", rarity: CardRarity::Epic, race: Race::Dragon, stats: CardStats { monster_count: 15, speed: 6, attack: 20, intelligence: 15, defense: 15, magic_defense: 10, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // === 遺跡敵（ノーマル） ===
    CardDef { id: 20, name: "ストーンゴーレム", rarity: CardRarity::Uncommon, race: Race::Giant, stats: CardStats { monster_count: 8, speed: 2, attack: 6, intelligence: 2, defense: 10, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 21, name: "ゴースト", rarity: CardRarity::Uncommon, race: Race::Undead, stats: CardStats { monster_count: 6, speed: 5, attack: 4, intelligence: 8, defense: 3, magic_defense: 8, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 22, name: "スケルトンナイト", rarity: CardRarity::Uncommon, race: Race::Undead, stats: CardStats { monster_count: 10, speed: 4, attack: 10, intelligence: 3, defense: 8, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 23, name: "コカトリス", rarity: CardRarity::Rare, race: Race::Beast, stats: CardStats { monster_count: 12, speed: 5, attack: 7, intelligence: 8, defense: 4, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 24, name: "ミミック", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 7, speed: 7, attack: 8, intelligence: 5, defense: 5, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 25, name: "ポイズンスパイダー", rarity: CardRarity::Uncommon, race: Race::Beast, stats: CardStats { monster_count: 5, speed: 6, attack: 6, intelligence: 4, defense: 3, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // === 遺跡敵（レア） ===
    CardDef { id: 30, name: "ダークウィザード", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 12, speed: 5, attack: 12, intelligence: 10, defense: 6, magic_defense: 8, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 31, name: "ガーゴイル", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 14, speed: 4, attack: 14, intelligence: 4, defense: 12, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 32, name: "シャドウアサシン", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 10, speed: 8, attack: 16, intelligence: 2, defense: 6, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 33, name: "フレイムスピリット", rarity: CardRarity::Rare, race: Race::Spirit, stats: CardStats { monster_count: 9, speed: 5, attack: 6, intelligence: 14, defense: 4, magic_defense: 10, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 34, name: "アイスエレメンタル", rarity: CardRarity::Rare, race: Race::Spirit, stats: CardStats { monster_count: 9, speed: 5, attack: 6, intelligence: 14, defense: 4, magic_defense: 10, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 35, name: "デスナイト", rarity: CardRarity::Epic, race: Race::Undead, stats: CardStats { monster_count: 16, speed: 4, attack: 12, intelligence: 6, defense: 14, magic_defense: 8, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 36, name: "ヒュドラ", rarity: CardRarity::Epic, race: Race::Dragon, stats: CardStats { monster_count: 14, speed: 4, attack: 14, intelligence: 12, defense: 10, magic_defense: 10, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 37, name: "ミノタウロス", rarity: CardRarity::Epic, race: Race::Giant, stats: CardStats { monster_count: 14, speed: 5, attack: 16, intelligence: 4, defense: 12, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // === 遺跡敵（レジェンダリー） ===
    CardDef { id: 40, name: "ニーズヘッグ", rarity: CardRarity::Legendary, race: Race::Dragon, stats: CardStats { monster_count: 30, speed: 4, attack: 25, intelligence: 18, defense: 20, magic_defense: 15, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 41, name: "ヴァンパイアロード", rarity: CardRarity::Legendary, race: Race::Undead, stats: CardStats { monster_count: 22, speed: 6, attack: 18, intelligence: 20, defense: 14, magic_defense: 18, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 42, name: "リッチ", rarity: CardRarity::Legendary, race: Race::Undead, stats: CardStats { monster_count: 22, speed: 5, attack: 15, intelligence: 25, defense: 12, magic_defense: 20, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 43, name: "タイタン", rarity: CardRarity::Legendary, race: Race::Giant, stats: CardStats { monster_count: 35, speed: 2, attack: 22, intelligence: 12, defense: 25, magic_defense: 18, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // =====================================================
    // === 収集可能カード（KC wiki 準拠で追加） ===
    // =====================================================

    // --- 獣族 (Beast) ---
    CardDef { id: 50, name: "バット", rarity: CardRarity::Common, race: Race::Beast, stats: CardStats { monster_count: 8, speed: 8, attack: 4, intelligence: 2, defense: 3, magic_defense: 2, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 51, name: "ジャイアントバット", rarity: CardRarity::Uncommon, race: Race::Beast, stats: CardStats { monster_count: 10, speed: 7, attack: 6, intelligence: 3, defense: 5, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 52, name: "ヴァンパイアバット", rarity: CardRarity::Uncommon, race: Race::Beast, stats: CardStats { monster_count: 12, speed: 7, attack: 7, intelligence: 4, defense: 5, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 53, name: "カマソッツ", rarity: CardRarity::Rare, race: Race::Beast, stats: CardStats { monster_count: 14, speed: 8, attack: 9, intelligence: 5, defense: 6, magic_defense: 5, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 54, name: "ボーゲスト", rarity: CardRarity::Uncommon, race: Race::Beast, stats: CardStats { monster_count: 11, speed: 5, attack: 7, intelligence: 4, defense: 6, magic_defense: 3, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 55, name: "ガイトラッシュ", rarity: CardRarity::Rare, race: Race::Beast, stats: CardStats { monster_count: 14, speed: 6, attack: 10, intelligence: 5, defense: 7, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 56, name: "レウクロコタ", rarity: CardRarity::Epic, race: Race::Beast, stats: CardStats { monster_count: 18, speed: 9, attack: 14, intelligence: 6, defense: 8, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 亜人族 (Demihuman) ---
    CardDef { id: 60, name: "ゴブリンアーチャー", rarity: CardRarity::Common, race: Race::Demihuman, stats: CardStats { monster_count: 6, speed: 4, attack: 5, intelligence: 2, defense: 2, magic_defense: 1, range: 3, cost: 1.0, occupation_power: 90 }, default_skills: None },
    CardDef { id: 61, name: "ゴブリンコック", rarity: CardRarity::Common, race: Race::Demihuman, stats: CardStats { monster_count: 10, speed: 4, attack: 5, intelligence: 5, defense: 5, magic_defense: 4, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 62, name: "ホブゴブリン", rarity: CardRarity::Uncommon, race: Race::Demihuman, stats: CardStats { monster_count: 12, speed: 5, attack: 7, intelligence: 5, defense: 8, magic_defense: 5, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 63, name: "オークアーマーナイト", rarity: CardRarity::Uncommon, race: Race::Demihuman, stats: CardStats { monster_count: 14, speed: 3, attack: 8, intelligence: 4, defense: 10, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 64, name: "ゴブリンソードマン", rarity: CardRarity::Uncommon, race: Race::Demihuman, stats: CardStats { monster_count: 12, speed: 6, attack: 9, intelligence: 5, defense: 6, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 65, name: "ホブゴブリンダークナイト", rarity: CardRarity::Rare, race: Race::Demihuman, stats: CardStats { monster_count: 16, speed: 5, attack: 12, intelligence: 4, defense: 11, magic_defense: 6, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 66, name: "ゴブリンプリンセス", rarity: CardRarity::Epic, race: Race::Demihuman, stats: CardStats { monster_count: 15, speed: 5, attack: 12, intelligence: 10, defense: 10, magic_defense: 8, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 魔族 (Demon) ---
    CardDef { id: 70, name: "レッサーデーモン", rarity: CardRarity::Common, race: Race::Demon, stats: CardStats { monster_count: 8, speed: 5, attack: 6, intelligence: 5, defense: 4, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 71, name: "サキュバス", rarity: CardRarity::Uncommon, race: Race::Demon, stats: CardStats { monster_count: 10, speed: 6, attack: 5, intelligence: 10, defense: 4, magic_defense: 8, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 72, name: "ナイトメア", rarity: CardRarity::Uncommon, race: Race::Demon, stats: CardStats { monster_count: 12, speed: 7, attack: 8, intelligence: 6, defense: 6, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 73, name: "アークデーモン", rarity: CardRarity::Rare, race: Race::Demon, stats: CardStats { monster_count: 16, speed: 5, attack: 13, intelligence: 8, defense: 10, magic_defense: 8, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 74, name: "リリス", rarity: CardRarity::Epic, race: Race::Demon, stats: CardStats { monster_count: 14, speed: 7, attack: 10, intelligence: 16, defense: 8, magic_defense: 14, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 竜族 (Dragon) ---
    CardDef { id: 80, name: "リンドヴルム", rarity: CardRarity::Common, race: Race::Dragon, stats: CardStats { monster_count: 8, speed: 4, attack: 7, intelligence: 4, defense: 5, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 81, name: "シーサーペント", rarity: CardRarity::Uncommon, race: Race::Dragon, stats: CardStats { monster_count: 12, speed: 5, attack: 8, intelligence: 6, defense: 7, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 82, name: "ファイアドレイク", rarity: CardRarity::Rare, race: Race::Dragon, stats: CardStats { monster_count: 14, speed: 5, attack: 11, intelligence: 9, defense: 8, magic_defense: 7, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 83, name: "バハムート", rarity: CardRarity::Epic, race: Race::Dragon, stats: CardStats { monster_count: 20, speed: 4, attack: 18, intelligence: 14, defense: 16, magic_defense: 12, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 巨人族 (Giant) ---
    CardDef { id: 90, name: "オーガ", rarity: CardRarity::Common, race: Race::Giant, stats: CardStats { monster_count: 10, speed: 3, attack: 7, intelligence: 2, defense: 6, magic_defense: 2, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 91, name: "サイクロプス", rarity: CardRarity::Uncommon, race: Race::Giant, stats: CardStats { monster_count: 14, speed: 3, attack: 9, intelligence: 3, defense: 9, magic_defense: 4, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 92, name: "アイアンゴーレム", rarity: CardRarity::Rare, race: Race::Giant, stats: CardStats { monster_count: 18, speed: 2, attack: 10, intelligence: 3, defense: 14, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 93, name: "ギガース", rarity: CardRarity::Epic, race: Race::Giant, stats: CardStats { monster_count: 22, speed: 3, attack: 16, intelligence: 5, defense: 18, magic_defense: 10, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 精霊族 (Spirit) ---
    CardDef { id: 100, name: "ウィスプ", rarity: CardRarity::Common, race: Race::Spirit, stats: CardStats { monster_count: 6, speed: 6, attack: 3, intelligence: 7, defense: 2, magic_defense: 6, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 101, name: "シルフ", rarity: CardRarity::Uncommon, race: Race::Spirit, stats: CardStats { monster_count: 8, speed: 7, attack: 4, intelligence: 9, defense: 3, magic_defense: 7, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 102, name: "ウンディーネ", rarity: CardRarity::Uncommon, race: Race::Spirit, stats: CardStats { monster_count: 10, speed: 5, attack: 5, intelligence: 10, defense: 5, magic_defense: 9, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 103, name: "イフリート", rarity: CardRarity::Rare, race: Race::Spirit, stats: CardStats { monster_count: 14, speed: 5, attack: 12, intelligence: 11, defense: 7, magic_defense: 8, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 104, name: "フェニックス", rarity: CardRarity::Epic, race: Race::Spirit, stats: CardStats { monster_count: 16, speed: 6, attack: 10, intelligence: 15, defense: 8, magic_defense: 14, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },

    // --- 不死族 (Undead) ---
    CardDef { id: 110, name: "ゾンビ", rarity: CardRarity::Common, race: Race::Undead, stats: CardStats { monster_count: 10, speed: 2, attack: 5, intelligence: 1, defense: 6, magic_defense: 2, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 111, name: "レイス", rarity: CardRarity::Uncommon, race: Race::Undead, stats: CardStats { monster_count: 8, speed: 5, attack: 4, intelligence: 8, defense: 3, magic_defense: 9, range: 2, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 112, name: "ワイト", rarity: CardRarity::Uncommon, race: Race::Undead, stats: CardStats { monster_count: 12, speed: 4, attack: 8, intelligence: 5, defense: 7, magic_defense: 5, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 113, name: "ドゥラハン", rarity: CardRarity::Rare, race: Race::Undead, stats: CardStats { monster_count: 14, speed: 5, attack: 11, intelligence: 4, defense: 10, magic_defense: 6, range: 1, cost: 1.5, occupation_power: 100 }, default_skills: None },
    CardDef { id: 114, name: "エルダーリッチ", rarity: CardRarity::Epic, race: Race::Undead, stats: CardStats { monster_count: 16, speed: 4, attack: 8, intelligence: 18, defense: 8, magic_defense: 16, range: 3, cost: 1.0, occupation_power: 90 }, default_skills: None },
];

/// カードIDからカード定義を取得
pub fn get_card(id: u32) -> Option<&'static CardDef> {
    CARDS.iter().find(|c| c.id == id)
}

fn normalize_card_name(name: &str) -> &str {
    match name.trim_end_matches(|c| c == 'A' || c == 'B' || c == 'C') {
        "カースドアーマー" | "呪われた鎧" => "ガーゴイル",
        "炎の精霊" => "フレイムスピリット",
        "氷の精霊" | "アイスエレメンタル" => "アイスエレメンタル",
        "毒蜘蛛" => "ポイズンスパイダー",
        "ファントム" => "ゴースト",
        "スライムキング" => "コカトリス",
        "クリスタルゴーレム" => "ミノタウロス",
        "ネクロマンサー" | "リッチロード" => "リッチ",
        "遺跡の守護者" => "ニーズヘッグ",
        "ドラゴンゾンビ" => "ヴァンパイアロード",
        "タイタンコロッサス" => "タイタン",
        "スライム" => "ゴブリン",
        "骸骨戦士" => "スケルトン",
        "オーガ" => "トロール",
        "ワイバーン" => "ドレイク",
        "トレジャーミミック" => "ミミック",
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

/// レアリティに応じたNPC用デフォルト（default_skills 未設定時）
fn fallback_npc_skill_data(c: &CardDef) -> SkillData {
    use crate::skills::SkillData;
    let (active, passive): (&'static str, Option<&'static str>) = match c.rarity {
        CardRarity::Common => ("power_smash", None),
        CardRarity::Uncommon => ("flash_cut", None),
        CardRarity::Rare => ("critical_edge", Some("power_aura")),
        CardRarity::Epic => ("heavy_impact", Some("battle_cry")),
        CardRarity::Legendary => ("armor_break", Some("intimidate")),
    };
    let skill2 = match c.race {
        Race::Beast => Some("swift_blade".to_string()),
        Race::Demihuman => Some("sharp_thrust".to_string()),
        Race::Demon => Some("monster_steal".to_string()),
        Race::Dragon => Some("blaze_edge".to_string()),
        Race::Giant => Some("whirlwind".to_string()),
        Race::Spirit => Some("heal_strike".to_string()),
        Race::Undead => Some("life_drain".to_string()),
    };
    SkillData {
        passive_id: passive.map(|s| s.to_string()),
        active_id: active.to_string(),
        unique_id: None,
        skill_level: 1,
        skill2_id: skill2,
        skill3_id: None,
        slot_levels: [1, 1, 1],
    }
}

/// カードIDに対応するデフォルトスキルを取得
pub fn get_card_skills(card_id: u32) -> SkillData {
    match get_card(card_id) {
        Some(c) if c.default_skills.is_some() => c.default_skills.clone().unwrap(),
        Some(c) => fallback_npc_skill_data(c),
        None => SkillData::default(),
    }
}
