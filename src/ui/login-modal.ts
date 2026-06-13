import type { GameMode } from "../config";
import { discoverAvailableModes } from "../network/server-discovery";
import {
  getSelectedPvpServerId,
  pvpServersPanelHtml,
  renderPvpServerList,
} from "./pvp-server-picker";

export interface LoginFormResult {
  username: string;
  password: string;
  register: boolean;
  mode: GameMode;
  pvpServerId?: string;
}

interface LoginModalOptions {
  error?: string;
  username?: string;
  mode?: GameMode;
  /** 起動中のサーバーモード */
  availableModes?: GameMode[];
}

let activeModal: HTMLDivElement | null = null;

function modeButtonHtml(mode: GameMode, label: string, selected: boolean): string {
  return `
    <button type="button" class="login-modal-mode-btn${selected ? " is-selected" : ""}" data-mode="${mode}">
      ${label}
    </button>
  `;
}

function pickInitialMode(available: GameMode[], preferred?: GameMode): GameMode {
  if (preferred === "pvp" || preferred === "pve") return preferred;
  if (available.includes("pve")) return "pve";
  return "pvp";
}

function getSelectedMode(overlay: HTMLElement): GameMode | undefined {
  const btn = overlay.querySelector<HTMLButtonElement>(".login-modal-mode-btn.is-selected");
  const mode = btn?.dataset.mode;
  return mode === "pve" || mode === "pvp" ? mode : undefined;
}

function wireModeButtons(overlay: HTMLElement): void {
  overlay.querySelectorAll<HTMLButtonElement>(".login-modal-mode-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      overlay.querySelectorAll(".login-modal-mode-btn").forEach((el) => {
        el.classList.remove("is-selected");
      });
      btn.classList.add("is-selected");
      updateServersPanel(overlay);
    });
  });
}

function updateServersPanel(overlay: HTMLElement): void {
  const panel = overlay.querySelector<HTMLElement>(".login-modal-servers");
  if (!panel) return;
  panel.hidden = getSelectedMode(overlay) !== "pvp";
}

export function showLoginModal(options: LoginModalOptions = {}): Promise<LoginFormResult | null> {
  if (activeModal) {
    activeModal.remove();
    activeModal = null;
    document.body.classList.remove("login-active");
  }

  let availableModes = [...(options.availableModes ?? [])];
  const initialMode = pickInitialMode(availableModes, options.mode);

  const modeButtons = [
    modeButtonHtml("pvp", "PVP", initialMode === "pvp"),
    modeButtonHtml("pve", "PVE", initialMode === "pve"),
  ];

  const serversPanel = pvpServersPanelHtml().replace(
    'class="login-modal-servers"',
    'class="login-modal-servers" hidden',
  );

  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "login-modal-overlay";
    overlay.innerHTML = `
      <form class="login-modal" autocomplete="on">
        <h2 class="login-modal-title">ログイン</h2>
        <fieldset class="login-modal-mode">
          <legend class="login-modal-mode-legend">プレイモード</legend>
          <div class="login-modal-mode-row">${modeButtons.join("")}</div>
        </fieldset>
        ${serversPanel}
        <label class="login-modal-field">
          <span>ユーザー名</span>
          <input name="username" type="text" required minlength="3" maxlength="32" autocomplete="username" />
        </label>
        <label class="login-modal-field">
          <span>パスワード</span>
          <input name="password" type="password" required minlength="4" autocomplete="current-password" />
        </label>
        <p class="login-modal-error" ${options.error ? "" : "hidden"}>${options.error ?? ""}</p>
        <div class="login-modal-actions">
          <button type="button" class="login-modal-cancel">キャンセル</button>
          <button type="submit" class="login-modal-submit">ログイン</button>
          <button type="button" class="login-modal-register">新規登録</button>
        </div>
      </form>
    `;

    const usernameInput = overlay.querySelector<HTMLInputElement>('input[name="username"]')!;
    const passwordInput = overlay.querySelector<HTMLInputElement>('input[name="password"]')!;
    const errorEl = overlay.querySelector<HTMLParagraphElement>(".login-modal-error")!;
    const cancelBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-cancel")!;
    const registerBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-register")!;
    const serverListEl = overlay.querySelector<HTMLElement>(".login-modal-server-list")!;
    const refreshBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-servers-refresh")!;

    if (options.username) usernameInput.value = options.username;

    const refreshServerList = async () => {
      const previous = getSelectedPvpServerId(serverListEl);
      refreshBtn.disabled = true;
      refreshBtn.classList.add("is-loading");
      try {
        availableModes = await discoverAvailableModes(true);
        renderPvpServerList(serverListEl, availableModes, previous);
      } finally {
        refreshBtn.disabled = false;
        refreshBtn.classList.remove("is-loading");
      }
    };

    wireModeButtons(overlay);
    renderPvpServerList(serverListEl, availableModes);
    refreshBtn.addEventListener("click", () => void refreshServerList());
    updateServersPanel(overlay);

    const close = (result: LoginFormResult | null) => {
      overlay.remove();
      if (activeModal === overlay) activeModal = null;
      document.body.classList.remove("login-active");
      resolve(result);
    };

    const form = overlay.querySelector<HTMLFormElement>(".login-modal")!;
    cancelBtn.addEventListener("click", () => close(null));
    overlay.addEventListener("click", (event) => {
      if (event.target === overlay) close(null);
    });

    const submit = (register: boolean) => {
      const username = usernameInput.value.trim();
      const password = passwordInput.value;
      const mode = getSelectedMode(overlay) ?? pickInitialMode(availableModes, options.mode);
      const pvpServerId = getSelectedPvpServerId(serverListEl);

      if (!username || !password) {
        errorEl.textContent = "ユーザー名とパスワードを入力してください。";
        errorEl.hidden = false;
        return;
      }
      if (mode === "pvp" && !pvpServerId) {
        errorEl.textContent = "PVP サーバーを選択するか、再取得してください。";
        errorEl.hidden = false;
        return;
      }

      close({
        username,
        password,
        register,
        mode,
        pvpServerId: mode === "pvp" ? pvpServerId : undefined,
      });
    };

    registerBtn.addEventListener("click", () => submit(true));
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      submit(false);
    });

    activeModal = overlay;
    document.body.classList.add("login-active");
    document.body.appendChild(overlay);
    usernameInput.focus();
  });
}
