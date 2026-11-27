const DEFAULTS = Object.freeze({ headers: { Accept: "application/json" } });
function buildRequest(url, headers = DEFAULTS.headers) {
  return { url, headers };
}
const authed = buildRequest("/me");
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public");
console.log(anon.headers);
