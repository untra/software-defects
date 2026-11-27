async function run(items) { await Promise.all(items.map(fetch)); }
