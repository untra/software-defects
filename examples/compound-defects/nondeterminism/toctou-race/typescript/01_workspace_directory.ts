import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const PREFIX = "workspace-";
const DIRECTORY = "reports";

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => { open = resolve; });
  return { promise, open };
}

async function initialize(path: string, ready: Promise<void>): Promise<void> {
  if (!existsSync(path)) {
    await ready;
    mkdirSync(path);
  }
}

async function main(): Promise<void> {
  const directory = mkdtempSync(join(tmpdir(), PREFIX));
  const path = join(directory, DIRECTORY);
  try {
    const ready = gate();
    const pending = initialize(path, ready.promise);
    mkdirSync(path);
    ready.open();
    await pending;
  } catch (error) {
    console.log("directory", (error as NodeJS.ErrnoException).code);
  } finally {
    rmSync(directory, { recursive: true });
  }
}

void main();
