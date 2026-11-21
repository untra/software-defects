const RESOURCE = "ledger";
const entries: string[] = [];
let complete!: () => void;
const pending = new Promise<void>(resolve => { complete = resolve; });
async function post() { await pending; entries.push("posted"); }
await navigator.locks.request(RESOURCE, async () => { const work = post(); complete(); await work; });
console.log("entries:", entries.length);
complete();
