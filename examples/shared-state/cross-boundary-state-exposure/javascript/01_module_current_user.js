const QUERY_LATENCY_MS = 10;
let currentUser = null;
async function loadAccount() {
    await new Promise((resolve) => setTimeout(resolve, QUERY_LATENCY_MS));
    return `account of ${currentUser}`;
}
async function handleRequest(user) {
    currentUser = user;
    return `${user} received ${await loadAccount()}`;
}
async function main() {
    const responses = await Promise.all([handleRequest("alice"), handleRequest("bob")]);
    responses.forEach((r) => console.log(r));
}
main();
