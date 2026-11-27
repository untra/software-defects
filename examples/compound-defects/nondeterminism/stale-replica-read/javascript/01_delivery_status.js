const INITIAL_STATUS = "packing";
const NEXT_STATUS = "shipped";
let primary = INITIAL_STATUS;
let replica = INITIAL_STATUS;
function gate() {
    let open = () => undefined;
    const promise = new Promise((resolve) => { open = resolve; });
    return { promise, open };
}
async function copy(ready) {
    await ready;
    replica = primary;
}
async function main() {
    const delivery = gate();
    primary = NEXT_STATUS;
    const pending = copy(delivery.promise);
    console.log("status", replica);
    delivery.open();
    await pending;
}
void main();
