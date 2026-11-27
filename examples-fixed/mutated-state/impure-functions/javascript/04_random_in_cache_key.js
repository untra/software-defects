const LOOKUPS = 3;
const responses = new Map();
let hits = 0;
function cacheKey(path, params) {
    return JSON.stringify({ path, params: Object.fromEntries(Object.entries(params).sort(([a], [b]) => a.localeCompare(b))) });
}
function fetchCached(path, params) {
    const key = cacheKey(path, params);
    const cached = responses.get(key);
    if (cached !== undefined) {
        hits++;
        return cached;
    }
    const body = `body of ${path}`;
    responses.set(key, body);
    return body;
}
for (let i = 0; i < LOOKUPS; i++)
    fetchCached("/products", { page: "1" });
console.log(`cache hits: ${hits} of ${LOOKUPS}`);
