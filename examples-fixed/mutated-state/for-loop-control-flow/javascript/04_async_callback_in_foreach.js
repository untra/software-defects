const SAVE_LATENCY_MS = 10;
async function save(id, store) {
    await new Promise((resolve) => setTimeout(resolve, SAVE_LATENCY_MS));
    store.push(id);
}
async function main() {
    const saved = [];
    for (const id of ["u1", "u2", "u3"]) {
        await save(id, saved);
    }
    console.log("saved before report:", saved.length);
}
main();
