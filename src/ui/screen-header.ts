/** ボトムメニューと各画面ヘッダーで共用するアイコン */
export const MENU_ICON_SRC = {
  home: "/icons/menu-home.png",
  map: "/icons/menu-map.png",
  formation: "/icons/menu-formation.png",
  alliance: "/icons/menu-alliance.png",
  market: "/icons/menu-market.png",
  history: "/icons/menu-history.png",
  status: "/icons/menu-status.png",
  inventory: "/icons/menu-inventory.png",
} as const;

export type MenuIconKey = keyof typeof MENU_ICON_SRC;

/** 画面ヘッダー用: メニューアイコン + タイトル */
export function renderScreenHeaderTitle(icon: MenuIconKey, title: string): string {
  return `<img src="${MENU_ICON_SRC[icon]}" alt="" class="screen-header-icon" aria-hidden="true"><span class="screen-header-text">${title}</span>`;
}
