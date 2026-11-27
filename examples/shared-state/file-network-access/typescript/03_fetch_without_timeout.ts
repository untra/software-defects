import { createServer } from "node:http";
import type { AddressInfo } from "node:net";

const WATCHDOG_MS = 300;

async function getExchangeRate(url: string): Promise<string> {
  const response = await fetch(url);
  return response.text();
}

const stalled = createServer(() => {});
stalled.listen(0, "127.0.0.1", async () => {
  const { port } = stalled.address() as AddressInfo;
  setTimeout(() => {
    console.log(`checkout still waiting on exchange rate after ${WATCHDOG_MS}ms`);
    process.exit(0);
  }, WATCHDOG_MS);
  console.log("rate:", await getExchangeRate(`http://127.0.0.1:${port}/rate`));
  process.exit(0);
});
