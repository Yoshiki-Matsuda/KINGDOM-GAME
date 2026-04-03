//! 遺跡システム - 敵定義と遺跡生成

use rand::Rng;
use serde::{Deserialize, Serialize};

/// 遺跡敵のタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuinEnemyType {
    // 基本敵
    Golem,
    Phantom,
    SkeletonKnight,
    TreasureMimic,
    DarkWizard,
    // 追加敵
    SlimeKing,
    CursedArmor,
    ShadowAssassin,
    FlameSpirit,
    IceElemental,
    PoisonSpider,
    StoneGargoyle,
    DeathKnight,
    Necromancer,
    CrystalGolem,
    ThunderHawk,
    EarthWyrm,
    VoidStalker,
    AncientMummy,
    DemonImp,
    // ボス敵
    RuinGuardian,
    DragonZombie,
    LichLord,
    TitanColossus,
}

impl RuinEnemyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuinEnemyType::Golem => "golem",
            RuinEnemyType::Phantom => "phantom",
            RuinEnemyType::SkeletonKnight => "skeleton_knight",
            RuinEnemyType::TreasureMimic => "treasure_mimic",
            RuinEnemyType::DarkWizard => "dark_wizard",
            RuinEnemyType::SlimeKing => "slime_king",
            RuinEnemyType::CursedArmor => "cursed_armor",
            RuinEnemyType::ShadowAssassin => "shadow_assassin",
            RuinEnemyType::FlameSpirit => "flame_spirit",
            RuinEnemyType::IceElemental => "ice_elemental",
            RuinEnemyType::PoisonSpider => "poison_spider",
            RuinEnemyType::StoneGargoyle => "stone_gargoyle",
            RuinEnemyType::DeathKnight => "death_knight",
            RuinEnemyType::Necromancer => "necromancer",
            RuinEnemyType::CrystalGolem => "crystal_golem",
            RuinEnemyType::ThunderHawk => "thunder_hawk",
            RuinEnemyType::EarthWyrm => "earth_wyrm",
            RuinEnemyType::VoidStalker => "void_stalker",
            RuinEnemyType::AncientMummy => "ancient_mummy",
            RuinEnemyType::DemonImp => "demon_imp",
            RuinEnemyType::RuinGuardian => "ruin_guardian",
            RuinEnemyType::DragonZombie => "dragon_zombie",
            RuinEnemyType::LichLord => "lich_lord",
            RuinEnemyType::TitanColossus => "titan_colossus",
        }
    }
}

/// 遺跡敵キャラクター定義
pub struct RuinEnemyDef {
    pub name: &'static str,
    pub monster_count: u32,
}

/// 遺跡敵の定義を取得
pub fn get_ruin_enemy(enemy_type: RuinEnemyType) -> RuinEnemyDef {
    match enemy_type {
        // 基本敵
        RuinEnemyType::Golem => RuinEnemyDef {
            name: "ゴーレム", monster_count: 15,
        },
        RuinEnemyType::Phantom => RuinEnemyDef {
            name: "ファントム", monster_count: 8,
        },
        RuinEnemyType::SkeletonKnight => RuinEnemyDef {
            name: "スケルトンナイト", monster_count: 10,
        },
        RuinEnemyType::TreasureMimic => RuinEnemyDef {
            name: "トレジャーミミック", monster_count: 5,
        },
        RuinEnemyType::DarkWizard => RuinEnemyDef {
            name: "ダークウィザード", monster_count: 12,
        },
        // 追加敵（低〜中級）
        RuinEnemyType::SlimeKing => RuinEnemyDef {
            name: "スライムキング", monster_count: 20,
        },
        RuinEnemyType::CursedArmor => RuinEnemyDef {
            name: "カースドアーマー", monster_count: 14,
        },
        RuinEnemyType::ShadowAssassin => RuinEnemyDef {
            name: "シャドウアサシン", monster_count: 7,
        },
        RuinEnemyType::FlameSpirit => RuinEnemyDef {
            name: "フレイムスピリット", monster_count: 9,
        },
        RuinEnemyType::IceElemental => RuinEnemyDef {
            name: "アイスエレメンタル", monster_count: 11,
        },
        RuinEnemyType::PoisonSpider => RuinEnemyDef {
            name: "ポイズンスパイダー", monster_count: 6,
        },
        RuinEnemyType::StoneGargoyle => RuinEnemyDef {
            name: "ストーンガーゴイル", monster_count: 13,
        },
        // 追加敵（中〜上級）
        RuinEnemyType::DeathKnight => RuinEnemyDef {
            name: "デスナイト", monster_count: 18,
        },
        RuinEnemyType::Necromancer => RuinEnemyDef {
            name: "ネクロマンサー", monster_count: 10,
        },
        RuinEnemyType::CrystalGolem => RuinEnemyDef {
            name: "クリスタルゴーレム", monster_count: 22,
        },
        RuinEnemyType::ThunderHawk => RuinEnemyDef {
            name: "サンダーホーク", monster_count: 8,
        },
        RuinEnemyType::EarthWyrm => RuinEnemyDef {
            name: "アースワーム", monster_count: 16,
        },
        RuinEnemyType::VoidStalker => RuinEnemyDef {
            name: "ヴォイドストーカー", monster_count: 12,
        },
        RuinEnemyType::AncientMummy => RuinEnemyDef {
            name: "エンシェントマミー", monster_count: 14,
        },
        RuinEnemyType::DemonImp => RuinEnemyDef {
            name: "デーモンインプ", monster_count: 6,
        },
        // ボス敵
        RuinEnemyType::RuinGuardian => RuinEnemyDef {
            name: "遺跡の守護者", monster_count: 25,
        },
        RuinEnemyType::DragonZombie => RuinEnemyDef {
            name: "ドラゴンゾンビ", monster_count: 30,
        },
        RuinEnemyType::LichLord => RuinEnemyDef {
            name: "リッチロード", monster_count: 20,
        },
        RuinEnemyType::TitanColossus => RuinEnemyDef {
            name: "タイタンコロッサス", monster_count: 40,
        },
    }
}

/// 遺跡の難易度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuinDifficulty {
    Normal,
    Rare,
    Legendary,
}

/// 遺跡ユニット編成
pub struct RuinFormation {
    pub name: &'static str,
    pub difficulty: RuinDifficulty,
    pub enemies: [RuinEnemyType; 3],
}

/// 全遺跡編成
pub const RUIN_FORMATIONS: &[RuinFormation] = &[
    // ノーマル遺跡
    RuinFormation { name: "石の番人", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::Golem, RuinEnemyType::Golem, RuinEnemyType::Golem] },
    RuinFormation { name: "亡霊の群れ", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::Phantom, RuinEnemyType::Phantom, RuinEnemyType::Phantom] },
    RuinFormation { name: "骸骨兵団", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::SkeletonKnight, RuinEnemyType::SkeletonKnight, RuinEnemyType::SkeletonKnight] },
    RuinFormation { name: "スライムの巣", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::SlimeKing, RuinEnemyType::TreasureMimic, RuinEnemyType::TreasureMimic] },
    RuinFormation { name: "蜘蛛の巣窟", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::PoisonSpider, RuinEnemyType::PoisonSpider, RuinEnemyType::PoisonSpider] },
    RuinFormation { name: "炎の回廊", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::FlameSpirit, RuinEnemyType::FlameSpirit, RuinEnemyType::Golem] },
    RuinFormation { name: "氷結の間", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::IceElemental, RuinEnemyType::IceElemental, RuinEnemyType::Phantom] },
    RuinFormation { name: "小悪魔の遊び場", difficulty: RuinDifficulty::Normal, enemies: [RuinEnemyType::DemonImp, RuinEnemyType::DemonImp, RuinEnemyType::DemonImp] },
    
    // レア遺跡
    RuinFormation { name: "闇の魔術師団", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::DarkWizard, RuinEnemyType::DarkWizard, RuinEnemyType::DarkWizard] },
    RuinFormation { name: "宝箱の罠", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::TreasureMimic, RuinEnemyType::TreasureMimic, RuinEnemyType::TreasureMimic] },
    RuinFormation { name: "混成警備隊", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::Golem, RuinEnemyType::SkeletonKnight, RuinEnemyType::Phantom] },
    RuinFormation { name: "闇と骨の同盟", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::DarkWizard, RuinEnemyType::SkeletonKnight, RuinEnemyType::SkeletonKnight] },
    RuinFormation { name: "呪われた武具庫", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::CursedArmor, RuinEnemyType::CursedArmor, RuinEnemyType::CursedArmor] },
    RuinFormation { name: "暗殺者の隠れ家", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::ShadowAssassin, RuinEnemyType::ShadowAssassin, RuinEnemyType::VoidStalker] },
    RuinFormation { name: "精霊の聖域", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::FlameSpirit, RuinEnemyType::IceElemental, RuinEnemyType::ThunderHawk] },
    RuinFormation { name: "ガーゴイルの塔", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::StoneGargoyle, RuinEnemyType::StoneGargoyle, RuinEnemyType::StoneGargoyle] },
    RuinFormation { name: "死者の墓所", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::AncientMummy, RuinEnemyType::SkeletonKnight, RuinEnemyType::Necromancer] },
    RuinFormation { name: "クリスタルの洞窟", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::CrystalGolem, RuinEnemyType::Golem, RuinEnemyType::Golem] },
    RuinFormation { name: "雷鳥の巣", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::ThunderHawk, RuinEnemyType::ThunderHawk, RuinEnemyType::ThunderHawk] },
    RuinFormation { name: "地底の主", difficulty: RuinDifficulty::Rare, enemies: [RuinEnemyType::EarthWyrm, RuinEnemyType::PoisonSpider, RuinEnemyType::PoisonSpider] },
    
    // ボス遺跡
    RuinFormation { name: "守護者の間", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::RuinGuardian, RuinEnemyType::Golem, RuinEnemyType::Golem] },
    RuinFormation { name: "暗黒の祭壇", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::DarkWizard, RuinEnemyType::DarkWizard, RuinEnemyType::RuinGuardian] },
    RuinFormation { name: "最深部の番人", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::RuinGuardian, RuinEnemyType::RuinGuardian, RuinEnemyType::RuinGuardian] },
    RuinFormation { name: "竜の墓場", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::DragonZombie, RuinEnemyType::DeathKnight, RuinEnemyType::DeathKnight] },
    RuinFormation { name: "死霊王の玉座", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::LichLord, RuinEnemyType::Necromancer, RuinEnemyType::Necromancer] },
    RuinFormation { name: "巨神の神殿", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::TitanColossus, RuinEnemyType::CrystalGolem, RuinEnemyType::CrystalGolem] },
    RuinFormation { name: "混沌の深淵", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::VoidStalker, RuinEnemyType::VoidStalker, RuinEnemyType::LichLord] },
    RuinFormation { name: "不死の軍団", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::DragonZombie, RuinEnemyType::LichLord, RuinEnemyType::DeathKnight] },
    RuinFormation { name: "終焉の間", difficulty: RuinDifficulty::Legendary, enemies: [RuinEnemyType::TitanColossus, RuinEnemyType::DragonZombie, RuinEnemyType::LichLord] },
];

/// ランダムな編成を取得
pub fn get_random_formation(difficulty: Option<RuinDifficulty>) -> &'static RuinFormation {
    let mut rng = rand::thread_rng();
    let filtered: Vec<&RuinFormation> = match difficulty {
        Some(d) => RUIN_FORMATIONS.iter().filter(|f| f.difficulty == d).collect(),
        None => RUIN_FORMATIONS.iter().collect(),
    };
    filtered[rng.gen_range(0..filtered.len())]
}

/// 遺跡の難易度をランダムに決定（確率: Normal 60%, Rare 30%, Boss 10%）
pub fn random_difficulty() -> RuinDifficulty {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();
    // 難易度分布: Normal 75%, Rare 20%, Boss 5%
    if roll < 0.75 {
        RuinDifficulty::Normal
    } else if roll < 0.95 {
        RuinDifficulty::Rare
    } else {
        RuinDifficulty::Legendary
    }
}

/// 遺跡情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuinInfo {
    pub formation_name: String,
    pub difficulty: RuinDifficulty,
    /// 敵のタイプID（3体）
    pub enemies: Vec<String>,
    /// 敵の表示名（3体）
    pub enemy_names: Vec<String>,
    /// 敵の魔獣数（3体）
    #[serde(alias = "enemy_energies")]
    pub enemy_monster_counts: Vec<u32>,
    /// 消滅時刻（Unix timestamp ms）
    #[serde(default)]
    pub expires_at: Option<u64>,
}

/// 難易度に応じた遺跡の有効時間（ミリ秒）
fn get_ruin_duration_ms(difficulty: RuinDifficulty) -> u64 {
    match difficulty {
        RuinDifficulty::Normal => 30 * 60 * 1000,  // 30分
        RuinDifficulty::Rare => 20 * 60 * 1000,    // 20分
        RuinDifficulty::Legendary => 10 * 60 * 1000,    // 10分
    }
}

/// 領土に遺跡を生成
pub fn generate_ruin(_territory_id: &str) -> RuinInfo {
    let difficulty = random_difficulty();
    let formation = get_random_formation(Some(difficulty));
    
    let enemies: Vec<String> = formation.enemies.iter().map(|e| e.as_str().to_string()).collect();
    let enemy_names: Vec<String> = formation.enemies.iter().map(|e| get_ruin_enemy(*e).name.to_string()).collect();
    let enemy_monster_counts: Vec<u32> = formation
        .enemies
        .iter()
        .map(|e| get_ruin_enemy(*e).monster_count)
        .collect();
    
    // 現在時刻 + 有効時間
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let expires_at = now_ms + get_ruin_duration_ms(difficulty);
    
    RuinInfo {
        formation_name: formation.name.to_string(),
        difficulty,
        enemies,
        enemy_names,
        enemy_monster_counts,
        expires_at: Some(expires_at),
    }
}
