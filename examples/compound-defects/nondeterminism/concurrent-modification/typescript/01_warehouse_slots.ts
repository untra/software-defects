import { Worker, isMainThread, parentPort, workerData } from "node:worker_threads";
import { once } from "node:events";

const FIRST_SLOT = 0;
const BEFORE = [1, 2, 3];
const AFTER = [10, 20, 30];
const READER = "reader";
const WRITER = "writer";

async function main(): Promise<void> {
  if (!isMainThread) {
    const slots = new Int32Array(workerData.buffer);
    if (workerData.role === WRITER) {
      slots.set(AFTER);
    } else {
      let total = 0;
      for (let index = 0; index < slots.length; index++) {
        total += slots[index];
        if (index === FIRST_SLOT) {
          parentPort!.postMessage("scanned");
          await once(parentPort!, "message");
        }
      }
      console.log("units", total);
    }
    parentPort!.close();
    return;
  }
  const buffer = new SharedArrayBuffer(BEFORE.length * Int32Array.BYTES_PER_ELEMENT);
  new Int32Array(buffer).set(BEFORE);
  const reader = new Worker(new URL(import.meta.url), { workerData: { buffer, role: READER } });
  const finished = once(reader, "exit");
  await once(reader, "message");
  const writer = new Worker(new URL(import.meta.url), { workerData: { buffer, role: WRITER } });
  await once(writer, "exit");
  reader.postMessage("continue");
  await finished;
}

void main();
