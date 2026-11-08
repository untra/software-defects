const REQUESTS = 50_000;

const MAX_ENTRIES = 100;
const responseCache = new Map<string, string>();

function render(requestId: string, path: string): string {
  const body = `<html>${path}</html>`;
  if (!responseCache.has(path) && responseCache.size >= MAX_ENTRIES) {
    const oldest = responseCache.keys().next().value;
    if (oldest !== undefined) responseCache.delete(oldest);
  }
  responseCache.set(path, body);
  return body;
}

for (let i = 0; i < REQUESTS; i++) render(`req-${i}`, "/home");
console.log("cached responses retained:", responseCache.size);
