import { createServer } from "node:http";
const WATCHDOG_MS = 300;
async function getExchangeRate(url) {
    const response = await fetch(url);
    return response.text();
}
const stalled = createServer(() => { });
stalled.listen(0, "127.0.0.1", async () => {
    const { port } = stalled.address();
    setTimeout(() => {
        console.log(`checkout still waiting on exchange rate after ${WATCHDOG_MS}ms`);
        process.exit(0);
    }, WATCHDOG_MS);
    console.log("rate:", await getExchangeRate(`http://127.0.0.1:${port}/rate`));
    process.exit(0);
});
