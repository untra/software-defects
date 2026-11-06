const ORIGIN_LATENCY_MS = 20;
const CONCURRENT_VIEWERS = 50;
let originFetches = 0;
const cache = new Map();
async function fetchFromOrigin(key) {
    originFetches++;
    await new Promise((resolve) => setTimeout(resolve, ORIGIN_LATENCY_MS));
    return `rendered ${key}`;
}
async function get(key) {
    const hit = cache.get(key);
    if (hit !== undefined)
        return hit;
    const value = await fetchFromOrigin(key);
    cache.set(key, value);
    return value;
}
async function main() {
    await Promise.all(Array.from({ length: CONCURRENT_VIEWERS }, () => get("/homepage")));
    console.log("origin fetches for one page:", originFetches);
}
main();
