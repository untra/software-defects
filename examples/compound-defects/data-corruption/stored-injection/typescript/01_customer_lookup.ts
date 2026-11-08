import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CUSTOMER_NAME = "O'Neil";

function main(): void {
  const directory = mkdtempSync(join(tmpdir(), "customers-"));
  const path = join(directory, "name.txt");
  try {
    writeFileSync(path, CUSTOMER_NAME);
    const name = readFileSync(path, "utf8");
    const query = "SELECT id FROM customers WHERE name = '" + name + "'";
    console.log(query);
  } finally {
    rmSync(directory, { recursive: true });
  }
}

main();
