const POLL_MS = 5;
const MOUNTED_MS = 30;
const OBSERVE_MS = 60;

let pollsAfterUnmount = 0;

class StockTicker {
  mounted = false;
  private timer?: ReturnType<typeof setInterval>;
  mount(): void {
    this.unmount();
    this.mounted = true;
    this.timer = setInterval(() => {
      if (!this.mounted) pollsAfterUnmount++;
    }, POLL_MS);
  }
  unmount(): void {
    clearInterval(this.timer);
    this.timer = undefined;
    this.mounted = false;
  }
}

const ticker = new StockTicker();
ticker.mount();
setTimeout(() => ticker.unmount(), MOUNTED_MS);
setTimeout(() => {
  console.log("ticker still polling after unmount:", pollsAfterUnmount > 0);
  process.exit(0);
}, MOUNTED_MS + OBSERVE_MS);
