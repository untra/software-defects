let shown = "";
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => {
        open = resolve;
    });
    return { promise, open };
}
async function load(id, ready) {
    await ready;
    shown = id;
}
async function main() {
    const earlier = gate();
    const later = gate();
    const first = load("alpha", earlier.promise);
    const second = load("beta", later.promise);
    later.open();
    await second;
    earlier.open();
    await first;
    console.log("shown", shown);
}
void main();
