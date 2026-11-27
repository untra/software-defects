const RATE_LIMIT = 5;
const USER_COUNT = 100;
const API_LATENCY_MS = 10;

let inFlight = 0;

async function fetchUser(id: number): Promise<string> {
  inFlight++;
  try {
    if (inFlight > RATE_LIMIT) throw new Error("429 Too Many Requests");
    await new Promise((resolve) => setTimeout(resolve, API_LATENCY_MS));
    return `user-${id}`;
  } finally {
    inFlight--;
  }
}

async function main() {
  const ids = Array.from({ length: USER_COUNT }, (_, i) => i);
  const results = await Promise.allSettled(ids.map(fetchUser));
  console.log("rate limited:", results.filter((r) => r.status === "rejected").length, "of", USER_COUNT);
}

main();
