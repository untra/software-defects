const PEERS = ["billing", "shipping"];
const started = new Set<string>();
let release: () => void = () => {};
const released = new Promise<void>((resolve) => { release = resolve; });

async function register(peer: string) {
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
