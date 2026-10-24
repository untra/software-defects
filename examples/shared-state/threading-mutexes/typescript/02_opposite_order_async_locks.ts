const WATCHDOG_MS = 200;
const IO_MS = 5;

class AsyncLock {
  private tail: Promise<void> = Promise.resolve();
  async acquire(): Promise<() => void> {
    let release!: () => void;
    const previous = this.tail;
    this.tail = new Promise<void>((resolve) => (release = resolve));
    await previous;
    return release;
  }
}

const inventoryLock = new AsyncLock();
const ledgerLock = new AsyncLock();
const io = () => new Promise((resolve) => setTimeout(resolve, IO_MS));

async function sellItem(): Promise<void> {
  const releaseInventory = await inventoryLock.acquire();
  await io();
  const releaseLedger = await ledgerLock.acquire();
  releaseLedger();
  releaseInventory();
}

async function refundItem(): Promise<void> {
  const releaseLedger = await ledgerLock.acquire();
  await io();
  const releaseInventory = await inventoryLock.acquire();
  releaseInventory();
  releaseLedger();
}

setTimeout(() => {
  console.log(`sale and refund still blocked after ${WATCHDOG_MS}ms`);
  process.exit(0);
}, WATCHDOG_MS);
Promise.all([sellItem(), refundItem()]).then(() => {
  console.log("both completed");
  process.exit(0);
});
