export {};
const RESOURCE = "ledger";
const WATCHDOG_MS = 100;
let completed = false;
setTimeout(() => console.log("completed:", completed), WATCHDOG_MS);
navigator.locks.request(RESOURCE, async () => {
    await navigator.locks.request(RESOURCE, () => { completed = true; });
});
