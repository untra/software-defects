const DEFAULT_HEADERS: Record<string, string> = { Accept: "application/json" };

function buildRequest(url: string, headers = DEFAULT_HEADERS) {
  return { url, headers: { ...headers } };
}

const authed = buildRequest("/me");
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public");
console.log(anon.headers);
