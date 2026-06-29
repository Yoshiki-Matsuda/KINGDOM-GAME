/**
 * ギルドメンバーカード — 表示専用
 */

interface GuildMember {
  id: string;
  name: string;
  rank?: string;
  merits?: string[];
  active_buff?: string;
}

interface Guild {
  id: string;
  name: string;
  members: GuildMember[];
}

interface AppState {
  guilds: Map<string, Guild>;
  members: Map<string, GuildMember>;
}

function getGlobal(): AppState {
  const store = (globalThis as any).__state as AppState | undefined;
  return store ?? { guilds: new Map(), members: new Map() };
}

export function renderGuildMemberCard(
  guildId: string,
  confirmLeaveGuild: (id: string) => void,
): string {
  const state = getGlobal();
  const guild = state.guilds.get(guildId) || {
    id: guildId,
    name: "Non-existent Guild",
    members: [],
  };

  const members = guild.members;
  let html = `<div class="p-4 bg-gray-900 rounded-md">
        <h2 class="text-lg font-bold mb-4">${guild.name} Members</h2>
        <div class="mb-4">
            <p class="text-sm text-gray-400">
            <button onClick={() => confirmLeaveGuild('${guildId}')} class="text-orange-500 hover:text-orange-400 font-medium">
                Leave Guild
            </button>
            </p>
        </div>
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-4">`;

  for (const m of members) {
    const member = state.members.get(m);
    const meritsHtml = member?.merits?.length > 0
      ? `<span class="px-1.5 py-0.5 bg-orange-900/40 text-orange-300 text-[10px] rounded">⭐ ${member.merits.join(", ")}</span>`
      : "";
    const buffHtml = member?.active_buff === "BERSERK"
      ? `<span class="px-1.5 py-0.5 bg-red-900/40 text-red-300 text-[10px] rounded">B</span>`
      : "";
    html += `
        <div class="group flex flex-col p-3 bg-gray-800 border border-gray-700 rounded hover:border-blue-500/50 transition-all">
            <div class="flex items-center gap-3 mb-2">
                <div class="w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white text-xs font-bold">${member?.rank || 'U'}</div>
                <div class="flex flex-col">
                    <div class="text-sm font-semibold">${member?.name}</div>
                    <div class="text-[10px] text-gray-500">${m}</div>
                </div>
            </div>
            <div class="flex flex-wrap gap-1">
                ${meritsHtml}
                ${buffHtml}
            </div>
        </div>`;
  }

  html += `</div>`;
  return html;
}
