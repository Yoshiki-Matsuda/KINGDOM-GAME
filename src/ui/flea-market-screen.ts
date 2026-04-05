/**
 * フリーマーケット画面 — 魔獣・アイテム・資源をゴールドで売買
 */

import { gameState, ws, render, setCurrentScreen } from "../store";
import { getBodyDisplayName } from "../game/characters";
import { getItem } from "../game/items";
import { getInventoryForState } from "../game/facility-selectors";
import type { MarketListing, MarketItemType, Action } from "../shared/game-state";
import { DEFAULT_PLAYER_ID, getPlayerOwnedCards } from "../shared/game-state";

let marketEl: HTMLDivElement;
let listingContainer: HTMLDivElement;
let currentTab: "browse" | "my_listings" | "sell" = "browse";

export function createFleaMarketElement(): HTMLDivElement {
  marketEl = document.createElement("div");
  marketEl.className = "flea-market-screen";
  marketEl.style.display = "none";

  marketEl.innerHTML = `
    <div class="market-header">
      <h2>フリーマーケット</h2>
      <div class="market-gold-display"></div>
    </div>
    <div class="market-tabs">
      <button type="button" class="market-tab is-active" data-tab="browse">出品一覧</button>
      <button type="button" class="market-tab" data-tab="my_listings">自分の出品</button>
      <button type="button" class="market-tab" data-tab="sell">出品する</button>
    </div>
    <div class="market-content"></div>
  `;

  listingContainer = marketEl.querySelector(".market-content") as HTMLDivElement;

  marketEl.querySelectorAll(".market-tab").forEach(btn => {
    btn.addEventListener("click", () => {
      currentTab = (btn as HTMLElement).dataset.tab as typeof currentTab;
      marketEl.querySelectorAll(".market-tab").forEach(b => b.classList.remove("is-active"));
      btn.classList.add("is-active");
      renderFleaMarket();
    });
  });

  return marketEl;
}

function getPlayerGold(): number {
  const player = gameState.players?.[DEFAULT_PLAYER_ID];
  return player?.resources?.gold ?? gameState.resources?.gold ?? 0;
}

function getListings(): MarketListing[] {
  return gameState.market_listings ?? [];
}

function sendMarketAction(action: Action): void {
  if (ws?.readyState !== WebSocket.OPEN) return;
  ws.send(JSON.stringify(action));
}

function describeItem(item: MarketItemType): string {
  switch (item.type) {
    case "card":
      return `[魔獣] ${getBodyDisplayName(item.card_id)}`;
    case "item": {
      const def = getItem(item.item_id);
      return `${def?.icon ?? ""} ${def?.name ?? item.item_id} x${item.count}`;
    }
    case "resource": {
      const names: Record<string, string> = { food: "食料", wood: "木材", stone: "石材", iron: "鉄" };
      return `${names[item.resource_type] ?? item.resource_type} x${item.amount}`;
    }
  }
}

function renderBrowseTab(): void {
  const listings = getListings().filter(l => l.seller_id !== DEFAULT_PLAYER_ID);
  const gold = getPlayerGold();

  if (listings.length === 0) {
    listingContainer.innerHTML = `<div class="market-empty">現在出品されているアイテムはありません</div>`;
    return;
  }

  listingContainer.innerHTML = listings.map(l => `
    <div class="market-listing-card">
      <div class="listing-item-info">${describeItem(l.item)}</div>
      <div class="listing-price">${l.price.toLocaleString()} G</div>
      <button type="button" class="listing-buy-btn${gold < l.price ? " disabled" : ""}"
              data-listing-id="${l.listing_id}"
              ${gold < l.price ? "disabled" : ""}>
        購入
      </button>
    </div>
  `).join("");

  listingContainer.querySelectorAll(".listing-buy-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const id = (btn as HTMLElement).dataset.listingId!;
      if (confirm("この出品を購入しますか？")) {
        sendMarketAction({ action: "buy_from_flea_market", listing_id: id });
      }
    });
  });
}

function renderMyListingsTab(): void {
  const listings = getListings().filter(l => l.seller_id === DEFAULT_PLAYER_ID);

  if (listings.length === 0) {
    listingContainer.innerHTML = `<div class="market-empty">出品中のアイテムはありません</div>`;
    return;
  }

  listingContainer.innerHTML = listings.map(l => `
    <div class="market-listing-card my-listing">
      <div class="listing-item-info">${describeItem(l.item)}</div>
      <div class="listing-price">${l.price.toLocaleString()} G</div>
      <button type="button" class="listing-cancel-btn" data-listing-id="${l.listing_id}">取消</button>
    </div>
  `).join("");

  listingContainer.querySelectorAll(".listing-cancel-btn").forEach(btn => {
    btn.addEventListener("click", () => {
      const id = (btn as HTMLElement).dataset.listingId!;
      sendMarketAction({ action: "cancel_flea_market_listing", listing_id: id });
    });
  });
}

function renderSellTab(): void {
  const ownedCards = getPlayerOwnedCards(gameState);
  const inventory = getInventoryForState(gameState);
  const player = gameState.players?.[DEFAULT_PLAYER_ID];
  const res = player?.resources ?? gameState.resources;

  listingContainer.innerHTML = `
    <div class="sell-form">
      <div class="sell-section">
        <h3>出品種別</h3>
        <select class="sell-type-select">
          <option value="card">魔獣</option>
          <option value="item">アイテム</option>
          <option value="resource">資源</option>
        </select>
      </div>

      <div class="sell-target-section">
        <h3>出品対象</h3>
        <select class="sell-target-select"></select>
        <div class="sell-amount-row" style="display:none">
          <label>数量</label>
          <input type="number" class="sell-amount-input" min="1" value="1" />
        </div>
      </div>

      <div class="sell-price-section">
        <h3>価格（ゴールド）</h3>
        <input type="number" class="sell-price-input" min="1" value="100" />
      </div>

      <button type="button" class="sell-confirm-btn">出品する</button>
    </div>
  `;

  const typeSelect = listingContainer.querySelector(".sell-type-select") as HTMLSelectElement;
  const targetSelect = listingContainer.querySelector(".sell-target-select") as HTMLSelectElement;
  const amountRow = listingContainer.querySelector(".sell-amount-row") as HTMLDivElement;
  const amountInput = listingContainer.querySelector(".sell-amount-input") as HTMLInputElement;
  const priceInput = listingContainer.querySelector(".sell-price-input") as HTMLInputElement;
  const confirmBtn = listingContainer.querySelector(".sell-confirm-btn") as HTMLButtonElement;

  function updateTargetOptions(): void {
    const type = typeSelect.value;
    targetSelect.innerHTML = "";
    amountRow.style.display = "none";

    if (type === "card") {
      if (ownedCards.length === 0) {
        targetSelect.innerHTML = `<option value="">出品できる魔獣がありません</option>`;
        return;
      }
      for (const cardId of ownedCards) {
        const name = getBodyDisplayName(cardId);
        const opt = document.createElement("option");
        opt.value = String(cardId);
        opt.textContent = name;
        targetSelect.appendChild(opt);
      }
    } else if (type === "item") {
      amountRow.style.display = "flex";
      if (inventory.length === 0) {
        targetSelect.innerHTML = `<option value="">アイテムがありません</option>`;
        return;
      }
      for (const inv of inventory) {
        const def = getItem(inv.item_id);
        const opt = document.createElement("option");
        opt.value = inv.item_id;
        opt.textContent = `${def?.name ?? inv.item_id} (所持: ${inv.count})`;
        targetSelect.appendChild(opt);
      }
      amountInput.max = String(inventory[0]?.count ?? 1);
    } else if (type === "resource") {
      amountRow.style.display = "flex";
      const types = [
        { id: "food", name: "食料", val: res?.food ?? 0 },
        { id: "wood", name: "木材", val: res?.wood ?? 0 },
        { id: "stone", name: "石材", val: res?.stone ?? 0 },
        { id: "iron", name: "鉄", val: res?.iron ?? 0 },
      ];
      for (const r of types) {
        const opt = document.createElement("option");
        opt.value = r.id;
        opt.textContent = `${r.name} (所持: ${r.val.toLocaleString()})`;
        targetSelect.appendChild(opt);
      }
      amountInput.max = String(types[0]?.val ?? 0);
    }
  }

  typeSelect.addEventListener("change", updateTargetOptions);

  targetSelect.addEventListener("change", () => {
    const type = typeSelect.value;
    if (type === "item") {
      const inv = inventory.find(i => i.item_id === targetSelect.value);
      amountInput.max = String(inv?.count ?? 1);
      amountInput.value = "1";
    } else if (type === "resource") {
      const vals: Record<string, number> = {
        food: res?.food ?? 0, wood: res?.wood ?? 0,
        stone: res?.stone ?? 0, iron: res?.iron ?? 0,
      };
      amountInput.max = String(vals[targetSelect.value] ?? 0);
      amountInput.value = "1";
    }
  });

  confirmBtn.addEventListener("click", () => {
    const type = typeSelect.value;
    const price = parseInt(priceInput.value, 10);
    if (!price || price < 1) { alert("価格を1以上に設定してください"); return; }

    let item: MarketItemType;

    if (type === "card") {
      const cardId = parseInt(targetSelect.value, 10);
      if (isNaN(cardId)) { alert("魔獣を選択してください"); return; }
      item = { type: "card", card_id: cardId };
    } else if (type === "item") {
      const itemId = targetSelect.value;
      const count = parseInt(amountInput.value, 10);
      if (!itemId || !count || count < 1) { alert("アイテムと数量を指定してください"); return; }
      item = { type: "item", item_id: itemId, count };
    } else {
      const resourceType = targetSelect.value;
      const amount = parseInt(amountInput.value, 10);
      if (!resourceType || !amount || amount < 1) { alert("資源と数量を指定してください"); return; }
      item = { type: "resource", resource_type: resourceType, amount };
    }

    sendMarketAction({ action: "list_on_flea_market", item, price });
    currentTab = "my_listings";
    marketEl.querySelectorAll(".market-tab").forEach(b => b.classList.remove("is-active"));
    marketEl.querySelector('[data-tab="my_listings"]')?.classList.add("is-active");
  });

  updateTargetOptions();
}

export function renderFleaMarket(): void {
  const goldDisplay = marketEl.querySelector(".market-gold-display") as HTMLDivElement;
  goldDisplay.textContent = `所持金: ${getPlayerGold().toLocaleString()} G`;

  switch (currentTab) {
    case "browse": renderBrowseTab(); break;
    case "my_listings": renderMyListingsTab(); break;
    case "sell": renderSellTab(); break;
  }
}

export function showFleaMarketScreen(): void {
  setCurrentScreen("market");
  renderFleaMarket();
  render();
}
