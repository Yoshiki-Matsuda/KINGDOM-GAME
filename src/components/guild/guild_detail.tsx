export function renderGuildDetail(guildId = '') {
    const name = guildId ? new URL(window.location.href).searchParams.get('guild') || 'Unknown Guild' : '';
    const joinButton = guildId && !userIsMember(guildId) ? `
    <button onClick={() => joinGuild('${guildId}')} class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded mt-2 w-full md:w-auto">
        Join Guild
    </button>` : '';

    document.write(`<div class="p-4 bg-gray-900 rounded-md">
        <h2 class="text-2xl font-bold mb-2">${name}</h2>
        ${joinButton}
        <div class="mt-6">
            <h3 class="text-lg font-bold mb-2">Guild Members</h3>
            <ul class="space-y-1">`);

    try {
        const members = await fetch(`/api/guild/${guildId}`).then(r => r.json());
        for (const m of members) {
            document.write(`<li class="p-2 bg-gray-800 rounded">${m.name} (${m.role})</li>`);
        }
    } catch (e) {
        console.error('failed to fetch guild members', e);
        document.write(`<div class="text-red-400 text-sm">Could not load member list. The guild may have been renamed or removed.</div>`);
    }

    document.write(`<div class="mt-6 bg-blue-900/40 border border-blue-800/50 rounded p-3 text-sm">
        <div class="flex items-center gap-2">
            <svg class="h-5 w-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.25 9L8.25 15M15 9L15 15M8.25 9H22M10.75 15H14.25"></path></svg>
            <div class="flex flex-col">
                <span class="font-medium">Guild Administration</span>
                <span class="text-xs text-gray-300">Guild leaders can set members and change the guild name.</span>
            </div>
        </div>
    `);
}
