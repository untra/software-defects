const DEFAULT_HEADERS: Readonly<Record<string, string>> = { Accept: "application/json" };
function buildRequest(url: string, headers: Record<string, string> = DEFAULT_HEADERS) {
  return { url, headers };
}
const authed = buildRequest("/me");
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public");
console.log(anon.headers);
