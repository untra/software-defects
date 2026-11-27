const cache = new Map(); function get(k) { if (!cache.has(k)) cache.set(k, fetch(k)); return cache.get(k); }
