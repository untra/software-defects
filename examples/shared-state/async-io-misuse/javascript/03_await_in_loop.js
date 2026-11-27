const PEERS = ["billing", "shipping"];
const started = new Set();
let release;
const released = new Promise((resolve) => { release = resolve; });

async function register(peer) {
  started.add(peer);
  await released;
}

async function coordinate() {
  for (const peer of PEERS) await register(peer);
}

async function main() {
  const work = coordinate();
  console.log("all peers registered:", started.size === PEERS.length);
  release();
  await work;
}
main();
