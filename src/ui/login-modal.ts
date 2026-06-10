export interface LoginFormResult {
  username: string;
  password: string;
  register: boolean;
}

interface LoginModalOptions {
  error?: string;
  username?: string;
}

let activeModal: HTMLDivElement | null = null;

export function showLoginModal(options: LoginModalOptions = {}): Promise<LoginFormResult | null> {
  if (activeModal) {
    activeModal.remove();
    activeModal = null;
  }

  return new Promise((resolve) => {
    const overlay = document.createElement("div");
    overlay.className = "login-modal-overlay";
    overlay.innerHTML = `
      <form class="login-modal" autocomplete="on">
        <h2 class="login-modal-title">ログイン</h2>
        <p class="login-modal-desc">ユーザー名とパスワードを入力してください。</p>
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

    const form = overlay.querySelector<HTMLFormElement>(".login-modal")!;
    const usernameInput = overlay.querySelector<HTMLInputElement>('input[name="username"]')!;
    const passwordInput = overlay.querySelector<HTMLInputElement>('input[name="password"]')!;
    const errorEl = overlay.querySelector<HTMLParagraphElement>(".login-modal-error")!;
    const cancelBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-cancel")!;
    const registerBtn = overlay.querySelector<HTMLButtonElement>(".login-modal-register")!;

    if (options.username) usernameInput.value = options.username;

    const close = (result: LoginFormResult | null) => {
      overlay.remove();
      if (activeModal === overlay) activeModal = null;
      resolve(result);
    };

    cancelBtn.addEventListener("click", () => close(null));
    overlay.addEventListener("click", (event) => {
      if (event.target === overlay) close(null);
    });

    const submit = (register: boolean) => {
      const username = usernameInput.value.trim();
      const password = passwordInput.value;
      if (!username || !password) {
        errorEl.textContent = "ユーザー名とパスワードを入力してください。";
        errorEl.hidden = false;
        return;
      }
      close({ username, password, register });
    };

    registerBtn.addEventListener("click", () => submit(true));
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      submit(false);
    });

    activeModal = overlay;
    document.body.appendChild(overlay);
    usernameInput.focus();
  });
}
