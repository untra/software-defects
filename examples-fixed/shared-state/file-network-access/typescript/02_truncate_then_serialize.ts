import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { open, rename, mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";

async function saveSettings(path: string, settings: Record<string, unknown>): Promise<void> {
  const encoded = JSON.stringify(settings);
  const directory = await mkdtemp(join(dirname(path), ".settings-"));
  const temporary = join(directory, "settings.json");
  try {
    const file = await open(temporary, "wx");
    try {
      await file.writeFile(encoded);
      await file.sync();
    } finally {
      await file.close();
    }
    await rename(temporary, path);
  } finally {
    await rm(directory, { recursive: true, force: true });
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
