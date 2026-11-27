import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";
import { once } from "node:events";

const COUNTER_SLOT = 0;
const COUNTER_SLOTS = 1;
const CLERKS = 2;
const COMMIT = "commit";

async function main(): Promise<void> {
  if (!isMainThread) {
    const counts = new Int32Array(workerData);
    let parcels = counts[COUNTER_SLOT];
    parentPort!.postMessage("loaded");
    await once(parentPort!, "message");
    parcels++;
    counts[COUNTER_SLOT] = parcels;
    parentPort!.close();
    return;
  }
  const buffer = new SharedArrayBuffer(COUNTER_SLOTS * Int32Array.BYTES_PER_ELEMENT);
  const workers = Array.from({ length: CLERKS }, () => new Worker(new URL(import.meta.url), { workerData: buffer }));
  const exits = workers.map((worker) => once(worker, "exit"));
  await Promise.all(workers.map((worker) => once(worker, "message")));
  workers.forEach((worker) => worker.postMessage(COMMIT));
  await Promise.all(exits);
  console.log("parcels", new Int32Array(buffer)[COUNTER_SLOT]);
}

void main();
