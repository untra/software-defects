const ORIGIN_LATENCY_MS = 20;
const CONCURRENT_VIEWERS = 50;

let originFetches = 0;
const cache = new Map<string, Promise<string>>();

async function fetchFromOrigin(key: string): Promise<string> {
  originFetches++;
  await new Promise((resolve) => setTimeout(resolve, ORIGIN_LATENCY_MS));
  return `rendered ${key}`;
}

async function get(key: string): Promise<string> {
  const hit = cache.get(key);
  if (hit !== undefined) return hit;
  const pending = fetchFromOrigin(key).catch(error => { cache.delete(key); throw error; });
  cache.set(key, pending);
  return pending;
}

async function main() {
  await Promise.all(Array.from({ length: CONCURRENT_VIEWERS }, () => get("/homepage")));
  console.log("origin fetches for one page:", originFetches);
}

main();
