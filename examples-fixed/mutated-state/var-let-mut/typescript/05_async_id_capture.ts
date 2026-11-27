async function apiCall(id: number): Promise<number> {
  await Promise.resolve();
  return id;
}

async function fetchItems(ids: number[]): Promise<number[]> {
  const promises = ids.map(async (id) => {
    const data = await apiCall(id);
    console.log(`Processed for ID: ${id}`);
    return data;
  });
  return Promise.all(promises);
}

async function main() {
  console.log((await fetchItems([1, 2])).join(","));
}

main();
