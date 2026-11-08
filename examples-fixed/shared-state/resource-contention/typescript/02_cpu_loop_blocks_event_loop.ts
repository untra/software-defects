import { createHash } from "node:crypto";

const HEARTBEAT_MS = 10;
const YIELD_INTERVAL = 1000;
const HASH_ROUNDS = 200_000;
const HEARTBEAT_TOLERANCE_MS = 50;

async function hashPassword(password: string): Promise<string> {
  let digest = password;
  for (let i = 0; i < HASH_ROUNDS; i++) {
    digest = createHash("sha256").update(digest).digest("hex");
    if (i % YIELD_INTERVAL === 0) await new Promise(resolve => setImmediate(resolve));
  }
  return digest;
}

const scheduled = Date.now();
setTimeout(() => {
  const delay = Date.now() - scheduled - HEARTBEAT_MS;
  console.log("heartbeat on time:", delay <= HEARTBEAT_TOLERANCE_MS);
}, HEARTBEAT_MS);

hashPassword("hunter2");
