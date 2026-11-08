const GATEWAY_LATENCY_MS = 50;
const TIMEOUT_MS = 20;
const SETTLE_MS = 100;

let charges = 0;

async function chargeCard(amount: number): Promise<string> {
  await new Promise((resolve) => setTimeout(resolve, GATEWAY_LATENCY_MS));
  charges++;
  return `charged ${amount}`;
}

function timeout(ms: number): Promise<never> {
  return new Promise((_, reject) => setTimeout(() => reject(new Error("timeout")), ms));
}

async function chargeWithTimeout(amount: number): Promise<string> {
  return Promise.race([chargeCard(amount), timeout(TIMEOUT_MS)]);
}

async function main() {
  try {
    await chargeWithTimeout(50);
  } catch {
    await chargeCard(50);
  }
  await new Promise((resolve) => setTimeout(resolve, SETTLE_MS));
  console.log("customer charged", charges, "times");
}

main();
