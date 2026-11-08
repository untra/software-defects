const CAPACITY = 1;
const START_TICK = 100;
const FINISH_TICK = 105;
let tick = START_TICK;
let occupied = 0;
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function report(ready) {
    if (occupied === CAPACITY) {
        console.log("report pool busy at", tick);
        return;
    }
    occupied++;
    const started = tick;
    try {
        await ready;
        console.log("slot held ticks", tick - started);
    }
    finally {
        occupied--;
    }
}
async function main() {
    const delivery = gate();
    const first = report(delivery.promise);
    tick = FINISH_TICK;
    await report(Promise.resolve());
    delivery.open();
    await first;
}
void main();
