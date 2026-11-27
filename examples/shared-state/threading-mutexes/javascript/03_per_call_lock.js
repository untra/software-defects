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
let count = 0;
async function record() {
    const lock = new AsyncLock();
    const release = await lock.acquire();
    try {
        const saved = count;
        await Promise.resolve();
        count = saved + 1;
    } finally { release(); }
}
await Promise.all([record(), record()]);
console.log("count:", count);
