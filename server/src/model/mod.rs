//! ゲーム状態・行動の共通データモデル（フロントと同一構造で JSON 化）
//!
//! 設計: サーバー権威・データ駆動。状態更新は純粋関数 `apply_action` のみ。
//! 最終形 PvPvE を想定し、owner_id でプレイヤー／中立を区別。

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::skills::SkillData;
use crate::items::InventoryItem;
use crate::ruins::RuinInfo;
use crate::cards::get_card;

mod action;
mod card_stats;
mod player;
mod progression;
mod state;
mod territory;
mod travel;
mod types;
mod world;

pub use action::*;
pub use card_stats::*;
pub use player::*;
pub use progression::*;
pub use state::*;
pub(crate) use territory::*;
pub use types::*;
pub use travel::{average_speed, travel_time_ms};
pub use world::migrate_legacy_neutral_enemies;
pub use world::generate_territories;
pub use world::resolve_terrain_seed;
pub(crate) use world::*;
