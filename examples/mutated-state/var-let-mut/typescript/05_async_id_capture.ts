async function apiCall(id: number): Promise<number> {
  await Promise.resolve();
  return id;
}

async function fetchItems(ids: number[]): Promise<number[]> {
  let currentId = 0;
  const promises = ids.map(async (id) => {
    currentId = id;
    const data = await apiCall(currentId);
    console.log(`Processed for ID: ${currentId}`);
    return data;
  });
  return Promise.all(promises);
}

async function main() {
  console.log((await fetchItems([1, 2])).join(","));
}

main();
