use super::*;

// ========== フリーマーケット ==========

const BASE_MARKET_FEE_PERCENT: u64 = 10;

fn calculate_market_fee(price: u64, facilities: &[crate::model::BuiltFacility]) -> u64 {
    let bonuses = crate::facilities::calculate_facility_bonuses(facilities);
    let reduction = bonuses.market_fee_reduction.min(BASE_MARKET_FEE_PERCENT as u32) as u64;
    let fee_percent = BASE_MARKET_FEE_PERCENT.saturating_sub(reduction);
    price * fee_percent / 100
}

pub(super) fn apply_list_on_flea_market(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    item: &MarketItemType,
    price: u64,
) -> GameState {
    if price == 0 {
        push_log(log, "価格は1以上に設定してください。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let item_desc = {
        let Some(player) = new_state.players.get_mut(actor_player_id) else {
            return state.clone();
        };
        match item {
            MarketItemType::Card { card_id } => {
                let idx = player.owned_cards.iter().position(|&id| id == *card_id);
                match idx {
                    Some(i) => {
                        player.owned_cards.remove(i);
                        if i < player.card_monster_counts.len() {
                            player.card_monster_counts.remove(i);
                        }
                        ensure_card_monster_counts(player);
                        let name = crate::cards::get_card(*card_id)
                            .map(|c| c.name.to_string())
                            .unwrap_or_else(|| format!("魔獣#{}", card_id));
                        name
                    }
                    None => {
                        push_log(log, "出品する魔獣を所持していません。".to_string());
                        return state.clone();
                    }
                }
            }
            MarketItemType::Item { item_id, count } => {
                let inv_item = player.inventory.iter_mut().find(|i| i.item_id == *item_id);
                match inv_item {
                    Some(inv) if inv.count >= *count => {
                        inv.count -= count;
                        if inv.count == 0 {
                            player.inventory.retain(|i| i.item_id != *item_id);
                        }
                        let name = crate::items::item_name(item_id);
                        format!("{}x{}", name, count)
                    }
                    _ => {
                        push_log(log, "出品するアイテムが足りません。".to_string());
                        return state.clone();
                    }
                }
            }
            MarketItemType::Resource { resource_type, amount } => {
                let has_enough = match resource_type.as_str() {
                    "food" => player.resources.food >= *amount,
                    "wood" => player.resources.wood >= *amount,
                    "stone" => player.resources.stone >= *amount,
                    "iron" => player.resources.iron >= *amount,
                    _ => false,
                };
                if !has_enough || *amount == 0 {
                    push_log(log, "出品する資源が足りません。".to_string());
                    return state.clone();
                }
                match resource_type.as_str() {
                    "food" => player.resources.food -= amount,
                    "wood" => player.resources.wood -= amount,
                    "stone" => player.resources.stone -= amount,
                    "iron" => player.resources.iron -= amount,
                    _ => {}
                }
                let type_name = match resource_type.as_str() {
                    "food" => "食料",
                    "wood" => "木材",
                    "stone" => "石材",
                    "iron" => "鉄",
                    _ => "不明",
                };
                format!("{}x{}", type_name, amount)
            }
        }
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let listing_id = format!("listing_{}_{}", now, new_state.market_listings.len());
    new_state.market_listings.push(MarketListing {
        listing_id,
        seller_id: actor_player_id.to_string(),
        item: item.clone(),
        price,
        listed_at: now,
    });

    let mut territories = new_state.territories.clone();
    if let Some(p) = new_state.players.get(actor_player_id) {
        sync_home_territory_body_counts_from_player(&mut territories, p);
    }
    new_state.territories = territories;

    push_log(log, format!("フリマに{}を{}Gで出品しました。", item_desc, price));
    new_state.log = log.clone();
    new_state
}

pub(super) fn apply_buy_from_flea_market(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    listing_id: &str,
) -> GameState {
    let listing_idx = state.market_listings.iter().position(|l| l.listing_id == listing_id);
    let Some(idx) = listing_idx else {
        push_log(log, "出品が見つかりません。".to_string());
        return state.clone();
    };
    let listing = &state.market_listings[idx];

    if listing.seller_id == actor_player_id {
        push_log(log, "自分の出品は購入できません。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let Some(buyer) = new_state.players.get_mut(actor_player_id) else {
        return state.clone();
    };

    if buyer.resources.gold < listing.price {
        push_log(log, "ゴールドが足りません。".to_string());
        return state.clone();
    }

    buyer.resources.gold -= listing.price;

    match &listing.item {
        MarketItemType::Card { card_id } => {
            buyer.owned_cards.push(*card_id);
            ensure_card_monster_counts(buyer);
        }
        MarketItemType::Item { item_id, count } => {
            if let Some(inv) = buyer.inventory.iter_mut().find(|i| i.item_id == *item_id) {
                inv.count += count;
            } else {
                buyer.inventory.push(crate::items::InventoryItem {
                    item_id: item_id.clone(),
                    count: *count,
                });
            }
        }
        MarketItemType::Resource { resource_type, amount } => {
            match resource_type.as_str() {
                "food" => buyer.resources.food += amount,
                "wood" => buyer.resources.wood += amount,
                "stone" => buyer.resources.stone += amount,
                "iron" => buyer.resources.iron += amount,
                _ => {}
            }
        }
    }

    let fee = calculate_market_fee(listing.price, &buyer.facilities);
    let seller_id = listing.seller_id.clone();
    let proceeds = listing.price.saturating_sub(fee);

    if let Some(seller) = new_state.players.get_mut(&seller_id) {
        seller.resources.gold += proceeds;
    }

    let item_desc = match &new_state.market_listings[idx].item {
        MarketItemType::Card { card_id } => {
            crate::cards::get_card(*card_id)
                .map(|c| c.name.to_string())
                .unwrap_or_else(|| format!("魔獣#{}", card_id))
        }
        MarketItemType::Item { item_id, count } => {
            format!("{}x{}", crate::items::item_name(item_id), count)
        }
        MarketItemType::Resource { resource_type, amount } => {
            let name = match resource_type.as_str() {
                "food" => "食料", "wood" => "木材", "stone" => "石材", "iron" => "鉄",
                _ => "不明",
            };
            format!("{}x{}", name, amount)
        }
    };

    new_state.market_listings.remove(idx);

    let mut territories = new_state.territories.clone();
    if let Some(p) = new_state.players.get(actor_player_id) {
        sync_home_territory_body_counts_from_player(&mut territories, p);
    }
    new_state.territories = territories;

    push_log(log, format!(
        "フリマで{}を{}Gで購入（手数料{}G）",
        item_desc, listing.price, fee
    ));
    new_state.log = log.clone();
    new_state
}

pub(super) fn apply_cancel_flea_market_listing(
    state: &GameState,
    log: &mut Vec<String>,
    actor_player_id: &str,
    listing_id: &str,
) -> GameState {
    let listing_idx = state.market_listings.iter().position(|l| l.listing_id == listing_id);
    let Some(idx) = listing_idx else {
        push_log(log, "出品が見つかりません。".to_string());
        return state.clone();
    };

    if state.market_listings[idx].seller_id != actor_player_id {
        push_log(log, "自分の出品のみ取り消せます。".to_string());
        return state.clone();
    }

    let mut new_state = state.clone();
    let listing = new_state.market_listings.remove(idx);

    {
        let Some(player) = new_state.players.get_mut(actor_player_id) else {
            return state.clone();
        };
        match &listing.item {
            MarketItemType::Card { card_id } => {
                player.owned_cards.push(*card_id);
                ensure_card_monster_counts(player);
            }
            MarketItemType::Item { item_id, count } => {
                if let Some(inv) = player.inventory.iter_mut().find(|i| i.item_id == *item_id) {
                    inv.count += count;
                } else {
                    player.inventory.push(crate::items::InventoryItem {
                        item_id: item_id.clone(),
                        count: *count,
                    });
                }
            }
            MarketItemType::Resource { resource_type, amount } => {
                match resource_type.as_str() {
                    "food" => player.resources.food += amount,
                    "wood" => player.resources.wood += amount,
                    "stone" => player.resources.stone += amount,
                    "iron" => player.resources.iron += amount,
                    _ => {}
                }
            }
        }
    }

    let mut territories = new_state.territories.clone();
    if let Some(p) = new_state.players.get(actor_player_id) {
        sync_home_territory_body_counts_from_player(&mut territories, p);
    }
    new_state.territories = territories;

    push_log(log, "出品を取り消しました。".to_string());
    new_state.log = log.clone();
    new_state
}
