const PEERS = ["billing", "shipping"];
const started = new Set<string>();
let release: () => void = () => {};
const released = new Promise<void>((resolve) => { release = resolve; });

async function register(peer: string) {
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
