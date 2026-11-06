class ClickTracker {
    clicks = 0;
    handle() {
        this.clicks++;
    }
}
const tracker = new ClickTracker();
const listeners = [];
listeners.push(tracker.handle.bind(tracker));
for (const listener of listeners) {
    try {
        listener();
    }
    catch (e) {
        console.log("error:", e.message);
    }
}
console.log("clicks:", tracker.clicks);
