/**
 * 同盟画面
 */

import { setCurrentScreen, render, gameState, ws } from "../store";
import { DEFAULT_PLAYER_ID, type Alliance } from "../shared/game-state";

let allianceEl: HTMLDivElement | null = null;

export function createAllianceElement(): HTMLDivElement {
  const el = document.createElement("div");
  el.className = "sub-screen alliance-screen";
  el.style.display = "none";
  allianceEl = el;
  return el;
}

export function showAllianceScreen(): void {
  setCurrentScreen("alliance");
  render();
}

function findPlayerAlliance(): Alliance | undefined {
  if (!gameState?.alliances) return undefined;
  return gameState.alliances.find(a => a.member_ids.includes(DEFAULT_PLAYER_ID));
}

export function renderAlliance(): void {
  if (!allianceEl) return;
  const alliance = findPlayerAlliance();

  allianceEl.innerHTML = `
    <div class="sub-screen-header">
      <h2>⚜️ 同盟</h2>
    </div>
    <div class="sub-screen-content">
      ${alliance
        ? `<div class="sub-screen-info">
             <div class="status-row">同盟名: ${alliance.name}</div>
             <div class="status-row">メンバー数: ${alliance.member_ids.length}</div>
             <div class="status-row">領地ポイント: ${alliance.territory_points}</div>
             <div class="status-row">同盟レベル: ${alliance.level ?? 1} / 累計寄付: ${alliance.donated_total ?? 0}</div>
             <div class="alliance-donate" style="margin-top:1rem">
               <span>食料・木材・石材・鉄を各50寄付（開発用）</span>
               <button type="button" class="sub-screen-btn" data-alliance-donate>寄付する</button>
             </div>
           </div>`
        : `<div class="sub-screen-empty">現在同盟に所属していません</div>
           <div class="sub-screen-actions">
             <button class="sub-screen-btn" disabled>同盟を設立（準備中）</button>
           </div>`
      }
    </div>
  `;

  const donateBtn = allianceEl.querySelector("[data-alliance-donate]");
  donateBtn?.addEventListener("click", () => {
    if (ws?.readyState !== WebSocket.OPEN) return;
    ws.send(
      JSON.stringify({
        action: "donate_alliance",
        food: 50,
        wood: 50,
        stone: 50,
        iron: 50,
      })
    );
  });
}
