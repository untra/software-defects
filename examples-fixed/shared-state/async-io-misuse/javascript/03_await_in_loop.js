const PEERS = ["billing", "shipping"];
const started = new Set();
let release;
const released = new Promise((resolve) => { release = resolve; });

async function register(peer) {
  started.add(peer);
  await released;
}

async function coordinate() {
  await Promise.all(PEERS.map(register));
}

async function main() {
  const work = coordinate();
  console.log("all peers registered:", started.size === PEERS.length);
  release();
  await work;
}
main();
