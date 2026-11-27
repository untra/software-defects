function buildRequest(url: string, headers: Record<string, string> = {}) {
  return { url, headers };
}
const authed = buildRequest("/me");
authed.headers["Authorization"] = "Bearer token";
const anon = buildRequest("/public");
console.log(anon.headers);
