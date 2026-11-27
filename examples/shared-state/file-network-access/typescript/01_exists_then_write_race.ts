import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CLAIM_LATENCY_MS = 10;

async function claimLeadership(lockPath: string, node: string): Promise<boolean> {
  if (!existsSync(lockPath)) {
    await new Promise((resolve) => setTimeout(resolve, CLAIM_LATENCY_MS));
    await writeFile(lockPath, node);
    return true;
  }
  return false;
}

async function main() {
  const dir = mkdtempSync(join(tmpdir(), "leader-"));
  const lockPath = join(dir, "leader.lock");
  const claims = await Promise.all(["node-a", "node-b", "node-c"].map((n) => claimLeadership(lockPath, n)));
  rmSync(dir, { recursive: true });
  console.log("nodes that believe they are leader:", claims.filter(Boolean).length);
}

main();
