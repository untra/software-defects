import { mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { open } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const ATTEMPTS = 20;

async function readManifest(path: string): Promise<unknown> {
  const file = await open(path, "r");
  try { return JSON.parse(await file.readFile("utf8")); }
  finally { await file.close(); }
}

async function main() {
  const dir = mkdtempSync(join(tmpdir(), "manifest-"));
  const path = join(dir, "manifest.json");
  writeFileSync(path, "{ not json");
  const before = readdirSync("/dev/fd").length;
  for (let i = 0; i < ATTEMPTS; i++) await readManifest(path).catch(() => {});
  console.log("file descriptors leaked:", readdirSync("/dev/fd").length - before);
  rmSync(dir, { recursive: true });
}

main();
