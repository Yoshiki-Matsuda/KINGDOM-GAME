use super::*;

pub use crate::cards::CardStats;

/// クライアントから送る行動。JSON の action は小文字スネーク（クライアントと一致）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Action {
    #[serde(rename = "end_turn")]
    EndTurn,
    /// 自領地に増援。owner_id == "player" の領地のみ。
    #[serde(rename = "deploy")]
    Deploy {
        territory_id: String,
        count: u32,
        /// 援軍の体ごとのモンスター数。未指定時は各1として扱う。
        #[serde(default)]
        monsters_per_body: Option<Vec<u32>>,
        /// 援軍の体ごとの表示名。
        #[serde(default)]
        body_names: Option<Vec<String>>,
    },
    /// 自領地から他領地へ攻撃。from は自領、to は隣接想定（現状は検証なし）。
    #[serde(rename = "attack")]
    Attack {
        from_territory_id: String,
        to_territory_id: String,
        count: u32,
        /// 攻撃側の体ごとのモンスター数（先頭から順に敵1体目・2体目…と戦闘）。未指定時は各体を1として扱う。
        #[serde(default)]
        monsters_per_body: Option<Vec<u32>>,
        /// 攻撃側の体ごとの表示名（戦闘ログ用）。未指定時は「味方ユニットN」。
        #[serde(default)]
        body_names: Option<Vec<String>>,
        /// 攻撃するユニットの表示名（ログの「〇〇が△△を攻撃」の〇〇）。未指定時は領地名。
        #[serde(default)]
        unit_name: Option<String>,
        /// 攻撃側の体ごとのSPEED。未指定時は各5として扱う。
        #[serde(default)]
        speed_per_body: Option<Vec<u32>>,
        /// 攻撃側の体ごとのスキルデータ。
        #[serde(default)]
        skills_per_body: Option<Vec<SkillData>>,
        /// 攻撃側の体ごとの全ステータス。
        #[serde(default)]
        stats_per_body: Option<Vec<CardStats>>,
        /// 編成に対応する所持魔獣スロットのインデックス（スタミナ・XP用。クライアントが付与）
        #[serde(default)]
        owned_card_indices: Option<Vec<usize>>,
    },
    /// 占領済み領地に前線基地を建設（KC準拠: 前線を拡大し施設スロット追加）
    #[serde(rename = "build_base")]
    BuildBase {
        territory_id: String,
    },
    /// KC準拠: 拠点内施設の建設/レベルアップ（同時1件キュー制限）
    #[serde(rename = "build_facility")]
    BuildFacility {
        facility_id: String,
        level: u8,
        /// 配置座標（ホームマップ上）。建設時間はサーバー側の施設定義から計算する
        #[serde(default)]
        position: Option<FacilityPosition>,
    },
    /// 魔獣合成（KC準拠: 素材魔獣を消費してベース魔獣のスキルLvアップ or スキル移植）
    #[serde(rename = "synthesize_card")]
    SynthesizeCard {
        base_card_index: usize,
        material_card_indices: Vec<usize>,
    },
    /// 所持魔獣スロットの魔獣を増産（食料消費・魔獣マスタの上限まで）
    #[serde(rename = "produce_monsters")]
    ProduceMonsters {
        card_index: usize,
        amount: u32,
    },
    /// 同盟結成（KC準拠）
    #[serde(rename = "create_alliance")]
    CreateAlliance {
        name: String,
    },
    /// 同盟参加
    #[serde(rename = "join_alliance")]
    JoinAlliance {
        alliance_id: String,
    },
    /// 同盟脱退
    #[serde(rename = "leave_alliance")]
    LeaveAlliance,
    /// フリマ出品
    #[serde(rename = "list_on_flea_market")]
    ListOnFleaMarket {
        item: MarketItemType,
        price: u64,
    },
    /// フリマ購入
    #[serde(rename = "buy_from_flea_market")]
    BuyFromFleaMarket {
        listing_id: String,
    },
    /// フリマ出品取消
    #[serde(rename = "cancel_flea_market_listing")]
    CancelFleaMarketListing {
        listing_id: String,
    },
    /// 探索を開始（占領済み領地・同時派遣数は exploration_level まで）
    #[serde(rename = "start_exploration")]
    StartExploration {
        territory_id: String,
        #[serde(default)]
        card_indices: Vec<usize>,
    },
    /// 探索結果を回収
    #[serde(rename = "collect_exploration")]
    CollectExploration {
        mission_id: String,
    },
    /// 同盟へ資源寄付（同盟レベル・寄付累計が増加）
    #[serde(rename = "donate_alliance")]
    DonateAlliance {
        food: u64,
        wood: u64,
        stone: u64,
        iron: u64,
    },
}
