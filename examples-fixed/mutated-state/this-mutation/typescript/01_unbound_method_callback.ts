class ClickTracker {
  clicks = 0;
  handle() {
    this.clicks++;
  }
}

const tracker = new ClickTracker();
const listeners: Array<() => void> = [];
listeners.push(tracker.handle.bind(tracker));
for (const listener of listeners) {
  try {
    listener();
  } catch (e) {
    console.log("error:", (e as Error).message);
  }
}
console.log("clicks:", tracker.clicks);
