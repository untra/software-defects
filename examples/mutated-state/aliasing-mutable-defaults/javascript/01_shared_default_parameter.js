const DEFAULT_HEADERS = { Accept: "application/json" };
function buildRequest(url, headers = DEFAULT_HEADERS) {
    return { url, headers };
}
const authed = buildRequest("/me");
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public");
console.log(anon.headers);
