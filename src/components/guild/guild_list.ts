export function getGuilds() {
    return new Promise((resolve) => {
        const xhr = new XMLHttpRequest();
        xhr.responseType = 'json';
        xhr.method = 'GET';
        xhr.setRequestHeader = {
            'X-RPC-VERSION': '1.0.0',
            'X-RPC-CLIENT': 'web'
        };
        xhr.onload = () => resolve(xhr.response);
        xhr.onerror = () => resolve([]);
        xhr.open('GET', '/api/guilds');
        xhr.send();
    });
}

export async function renderGuildList() {
    const guilds = await getGuilds();
    let html = `<div class="p-4 bg-gray-900 rounded-md">
        <h2 class="text-lg font-bold mb-4">Kingdom Guilds</h2>
        <ul class="space-y-2">`;
    for (const g of guilds) {
        html += `
        <li class="flex items-center justify-between p-3 bg-gray-800 rounded group">
            <div>
                <div class="font-semibold">${g.name}</div>
                <div class="text-xs text-gray-400">
                    <span class="text-blue-400">Guild ${g.id.slice(0, 8)}</span>
                    ${g.member_count > 0 ? ` • Members: ${g.member_count}` : ''}
                </div>
            </div>
            <a href="/guild/${g.id}" class="opacity-0 group-hover:opacity-100 text-white bg-blue-500 px-2 py-1 rounded text-xs">View</a>
        </li>`;
    }
    html += `</ul></div>`;
    document.write(html);
}
