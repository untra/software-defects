const SAVE_LATENCY_MS = 10;

async function save(id: string, store: string[]): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, SAVE_LATENCY_MS));
  store.push(id);
}

async function main() {
  const saved: string[] = [];
  ["u1", "u2", "u3"].forEach(async (id) => {
    await save(id, saved);
  });
  console.log("saved before report:", saved.length);
}

main();
