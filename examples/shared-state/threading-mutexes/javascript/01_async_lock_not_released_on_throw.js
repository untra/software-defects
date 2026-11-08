const WATCHDOG_MS = 200;
class AsyncLock {
    tail = Promise.resolve();
    async acquire() {
        let release;
        const previous = this.tail;
        this.tail = new Promise((resolve) => (release = resolve));
        await previous;
        return release;
    }
}
const ledgerLock = new AsyncLock();
async function postEntry(amount) {
    const release = await ledgerLock.acquire();
    if (amount < 0)
        throw new Error("negative amount");
    release();
}
async function main() {
    await postEntry(-5).catch((e) => console.log("rejected:", e.message));
    setTimeout(() => {
        console.log(`next entry still waiting for the ledger lock after ${WATCHDOG_MS}ms`);
        process.exit(0);
    }, WATCHDOG_MS);
    await postEntry(10);
    console.log("entry posted");
    process.exit(0);
}
main();
