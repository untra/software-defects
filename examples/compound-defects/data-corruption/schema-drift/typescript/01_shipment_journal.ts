import { mkdtempSync, appendFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const JOURNAL_VERSION = 2;
const WAREHOUSE_WEIGHT_GRAMS = 500;
const KIOSK_WEIGHT_KILOGRAMS = 2;

function warehouse(path: string): void {
  const parcel = { version: JOURNAL_VERSION, weightGrams: WAREHOUSE_WEIGHT_GRAMS };
  appendFileSync(path, JSON.stringify(parcel) + "\n");
}

function kiosk(path: string): void {
  const parcel = { weight: KIOSK_WEIGHT_KILOGRAMS };
  appendFileSync(path, JSON.stringify(parcel) + "\n");
}

function main(): void {
  const directory = mkdtempSync(join(tmpdir(), "shipments-"));
  const path = join(directory, "parcels.jsonl");
  try {
    warehouse(path);
    kiosk(path);
    const rows = readFileSync(path, "utf8").trim().split("\n");
    for (const row of rows) {
      const parcel = JSON.parse(row);
      console.log("grams", parcel.weightGrams);
    }
  } finally {
    rmSync(directory, { recursive: true });
  }
}

main();
