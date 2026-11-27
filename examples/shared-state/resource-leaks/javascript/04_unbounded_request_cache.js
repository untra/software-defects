const REQUESTS = 50_000;
const responseCache = new Map();
function render(requestId, path) {
    const body = `<html>${path}</html>`;
    responseCache.set(requestId, body);
    return body;
}
for (let i = 0; i < REQUESTS; i++)
    render(`req-${i}`, "/home");
console.log("cached responses retained:", responseCache.size);
