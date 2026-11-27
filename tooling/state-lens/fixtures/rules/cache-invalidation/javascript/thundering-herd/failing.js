const cache = new Map(); async function get(k) { if (cache.has(k)) return cache.get(k); const v = await fetch(k); cache.set(k, v); return v; }
