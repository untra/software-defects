import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { open } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

async function saveSettings(path: string, settings: Record<string, unknown>): Promise<void> {
  const file = await open(path, "w");
  try {
    await file.write(JSON.stringify(settings));
  } finally {
    await file.close();
  }
}

async function main() {
  const dir = mkdtempSync(join(tmpdir(), "settings-"));
  const path = join(dir, "settings.json");
  writeFileSync(path, JSON.stringify({ theme: "dark", quotaBytes: 1024 }));
  try {
    await saveSettings(path, { theme: "dark", quotaBytes: 2n ** 40n });
  } catch (e) {
    console.log("save failed:", (e as Error).message);
  }
  console.log("settings on disk now:", JSON.stringify(readFileSync(path, "utf8")));
  rmSync(dir, { recursive: true });
}

main();
