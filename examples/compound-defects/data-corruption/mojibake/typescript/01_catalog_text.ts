import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CATALOG_NAME = "café";
const WRITER_ENCODING = "latin1";
const READER_ENCODING = "utf8";
const catalog = new Map<string, string>();

function main(): void {
  const directory = mkdtempSync(join(tmpdir(), "catalog-"));
  const path = join(directory, "name.txt");
  try {
    writeFileSync(path, CATALOG_NAME, WRITER_ENCODING);
    const name = readFileSync(path, READER_ENCODING);
    catalog.set("coffee", name);
    console.log("name", catalog.get("coffee"));
  } finally {
    rmSync(directory, { recursive: true });
  }
}

main();
