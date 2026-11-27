const cache = new Map(); async function get(k) { await fetch(k); return cache.get(k); }
