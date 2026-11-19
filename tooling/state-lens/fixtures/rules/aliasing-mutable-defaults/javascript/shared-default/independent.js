const DEFAULT_HEADERS = { Accept: "application/json" };
function buildRequest(url, headers = DEFAULT_HEADERS) {
  return { url, headers };
}
const authed = buildRequest("/me", { Accept: "application/json" });
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public", { Accept: "text/plain" });
console.log(anon.headers);
