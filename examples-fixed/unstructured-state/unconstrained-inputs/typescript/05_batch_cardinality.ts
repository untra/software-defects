const MAX_BATCH_ITEMS = 3;

function importRows(rows: string[]) {
  if (rows.length > MAX_BATCH_ITEMS) {
    throw new Error("batch rejected");
  }
  let processed = 0;
  for (const row of rows) {
    processed += 1;
  }
  return processed;
}

try {
  console.log("processed", importRows(["a", "b", "c", "d"]));
} catch (error) {
  console.log(error instanceof Error ? error.message : "batch rejected");
}
