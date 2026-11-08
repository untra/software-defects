const QUERY_LATENCY_MS = 10;
async function loadAccount(user) {
    await new Promise((resolve) => setTimeout(resolve, QUERY_LATENCY_MS));
    return `account of ${user}`;
}
async function handleRequest(user) {
    return `${user} received ${await loadAccount(user)}`;
}
async function main() {
    const responses = await Promise.all([handleRequest("alice"), handleRequest("bob")]);
    responses.forEach((r) => console.log(r));
}
main();
