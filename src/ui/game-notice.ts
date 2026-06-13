let noticeEl: HTMLParagraphElement | null = null;
let hideTimer: number | null = null;

export function createGameNoticeElement(): HTMLParagraphElement {
  noticeEl = document.createElement("p");
  noticeEl.className = "game-notice";
  noticeEl.hidden = true;
  noticeEl.setAttribute("role", "status");
  return noticeEl;
}

export function showGameNotice(message: string, durationMs = 6000): void {
  if (!noticeEl) return;
  if (hideTimer !== null) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
  noticeEl.textContent = message;
  noticeEl.hidden = false;
  hideTimer = window.setTimeout(() => {
    if (noticeEl) noticeEl.hidden = true;
    hideTimer = null;
  }, durationMs);
}

export function clearGameNotice(): void {
  if (!noticeEl) return;
  if (hideTimer !== null) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
  noticeEl.hidden = true;
}
