async function run(items, other) { if (other.length < 2) await Promise.all(items.map(fetch)); }
