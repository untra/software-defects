const RESOURCE = "ledger";
const entries = [];
let complete;
const pending = new Promise(resolve => { complete = resolve; });
async function post() { await pending; entries.push("posted"); }
await navigator.locks.request(RESOURCE, async () => { const work = post(); complete(); await work; });
console.log("entries:", entries.length);
complete();
