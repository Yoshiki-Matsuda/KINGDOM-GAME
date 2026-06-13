import { PVP_SERVERS, type GameMode } from "../config";
import { discoverAvailableModes } from "../network/server-discovery";

export function getSelectedPvpServerId(listEl: HTMLElement): string | undefined {
  return listEl.querySelector<HTMLButtonElement>(".login-modal-server.is-selected")?.dataset.serverId;
}

export function wireServerButtons(listEl: HTMLElement): void {
  listEl.querySelectorAll<HTMLButtonElement>(".login-modal-server").forEach((btn) => {
    btn.addEventListener("click", () => {
      listEl.querySelectorAll(".login-modal-server").forEach((el) => {
        el.classList.remove("is-selected");
      });
      btn.classList.add("is-selected");
    });
  });
}

export function renderPvpServerList(
  listEl: HTMLElement,
  available: GameMode[],
  keepServerId?: string,
): void {
  const pvpUp = available.includes("pvp");
  const entries = pvpUp ? PVP_SERVERS : [];

  if (entries.length === 0) {
    listEl.innerHTML = `<p class="login-modal-server-empty">起動中のサーバーがありません</p>`;
    return;
  }

  const keepIndex = keepServerId ? entries.findIndex((s) => s.id === keepServerId) : 0;
  const selectedIndex = keepIndex >= 0 ? keepIndex : 0;

  listEl.innerHTML = entries
    .map(
      (server, index) => `
        <button type="button" class="login-modal-server${index === selectedIndex ? " is-selected" : ""}" data-server-id="${server.id}">
          ${server.label}
        </button>
      `,
    )
    .join("");

  wireServerButtons(listEl);
}

export function pvpServersPanelHtml(): string {
  return `
    <div class="login-modal-servers">
      <div class="login-modal-servers-header">
        <p class="login-modal-servers-title">サーバー</p>
        <button type="button" class="login-modal-servers-refresh" title="サーバー一覧を再取得" aria-label="サーバー一覧を再取得">↻</button>
      </div>
      <div class="login-modal-server-list"></div>
    </div>
  `;
}

let activePicker: HTMLDivElement | null = null;

/** PVP サーバー選択ダイアログ。選択した server id、キャンセル時 null */
export function showPvpServerPickerDialog(options?: {
  error?: string;
  initialServerId?: string;
}): Promise<string | null> {
  if (activePicker) {
    activePicker.remove();
    activePicker = null;
  }

  let availableModes: GameMode[] = [];

  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "pvp-server-dialog-overlay";
    overlay.innerHTML = `
      <div class="login-modal pvp-server-dialog" role="dialog" aria-labelledby="pvp-server-dialog-title">
        <h2 class="login-modal-title" id="pvp-server-dialog-title">PVP サーバー</h2>
        ${pvpServersPanelHtml()}
        <p class="login-modal-error" ${options?.error ? "" : "hidden"}>${options?.error ?? ""}</p>
        <div class="login-modal-actions">
          <button type="button" class="login-modal-cancel">キャンセル</button>
          <button type="button" class="pvp-server-dialog-connect login-modal-submit">接続</button>
        </div>
      </div>
    `;

    const serverListEl = overlay.querySelector<HTMLElement>(".login-modal-server-list")!;
    const refreshBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-servers-refresh")!;
    const errorEl = overlay.querySelector<HTMLParagraphElement>(".login-modal-error")!;
    const cancelBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-cancel")!;
    const connectBtn = overlay.querySelector<HTMLButtonElement>(".pvp-server-dialog-connect")!;

    const close = (serverId: string | null) => {
      overlay.remove();
      if (activePicker === overlay) activePicker = null;
      resolve(serverId);
    };

    const refreshServerList = async () => {
      const previous = getSelectedPvpServerId(serverListEl);
      refreshBtn.disabled = true;
      refreshBtn.classList.add("is-loading");
      try {
        availableModes = await discoverAvailableModes(true);
        renderPvpServerList(serverListEl, availableModes, previous ?? options?.initialServerId);
      } finally {
        refreshBtn.disabled = false;
        refreshBtn.classList.remove("is-loading");
      }
    };

    void (async () => {
      availableModes = await discoverAvailableModes();
      renderPvpServerList(serverListEl, availableModes, options?.initialServerId);
    })();

    refreshBtn.addEventListener("click", () => void refreshServerList());
    cancelBtn.addEventListener("click", () => close(null));
    overlay.addEventListener("click", (event) => {
      if (event.target === overlay) close(null);
    });
    connectBtn.addEventListener("click", () => {
      const serverId = getSelectedPvpServerId(serverListEl);
      if (!serverId) {
        errorEl.textContent = "サーバーを選択するか、再取得してください。";
        errorEl.hidden = false;
        return;
      }
      close(serverId);
    });

    activePicker = overlay;
    document.body.appendChild(overlay);
  });
}

export function pvpServerLabel(serverId: string): string | undefined {
  return PVP_SERVERS.find((s) => s.id === serverId)?.label;
}
