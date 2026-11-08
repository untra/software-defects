import { setTimeout as delay } from "node:timers/promises";

const GATEWAY_LATENCY_MS = 50;
const TIMEOUT_MS = 20;
const SETTLE_MS = 100;

let charges = 0;

async function chargeCard(amount: number, signal?: AbortSignal): Promise<string> {
  await delay(GATEWAY_LATENCY_MS, undefined, { signal });
  signal?.throwIfAborted();
  charges++;
  return `charged ${amount}`;
}

function timeout(ms: number): Promise<never> {
  return new Promise((_, reject) => setTimeout(() => reject(new Error("timeout")), ms));
}

async function chargeWithTimeout(amount: number): Promise<string> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(new Error("timeout")), TIMEOUT_MS);
  try { return await chargeCard(amount, controller.signal); }
  finally { clearTimeout(timer); }
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
