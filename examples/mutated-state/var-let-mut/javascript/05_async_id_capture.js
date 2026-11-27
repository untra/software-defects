async function apiCall(id) {
    await Promise.resolve();
    return id;
}
async function fetchItems(ids) {
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
