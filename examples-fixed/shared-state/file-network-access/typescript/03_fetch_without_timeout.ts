import { createServer } from "node:http";
import type { AddressInfo } from "node:net";

const REQUEST_TIMEOUT_MS = 50;
const WATCHDOG_MS = 300;

async function getExchangeRate(url: string): Promise<string> {
  const response = await fetch(url, { signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS) });
  return response.text();
}

const stalled = createServer(() => {});
stalled.listen(0, "127.0.0.1", async () => {
  const { port } = stalled.address() as AddressInfo;
  const watchdog = setTimeout(() => {
    console.log(`checkout still waiting on exchange rate after ${WATCHDOG_MS}ms`);
    process.exit(0);
  }, WATCHDOG_MS);
  try { console.log("rate:", await getExchangeRate(`http://127.0.0.1:${port}/rate`)); }
  catch { console.log("exchange rate timed out"); }
  finally { clearTimeout(watchdog); stalled.closeAllConnections(); stalled.close(); }
  process.exit(0);
});
