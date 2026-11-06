const LATENCY_BY_QUERY = { a: 30, ab: 20, abc: 5 };
let generation = 0;
let visibleResults = [];
async function search(query) {
    await new Promise((resolve) => setTimeout(resolve, LATENCY_BY_QUERY[query]));
    return [`${query}-result-1`, `${query}-result-2`];
}
async function onKeystroke(query) {
    const requestGeneration = ++generation;
    const results = await search(query);
    if (requestGeneration === generation)
        visibleResults = results;
}
async function main() {
    await Promise.all(["a", "ab", "abc"].map(onKeystroke));
    console.log("search box: abc, showing:", visibleResults);
}
main();
