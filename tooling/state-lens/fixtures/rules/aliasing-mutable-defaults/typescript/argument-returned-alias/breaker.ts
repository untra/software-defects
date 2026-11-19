const signed = { Authorization: "Bearer secret" };
function normalize(headers: Record<string, string>) {
  return headers;
}
let publicHeaders = normalize(signed);
publicHeaders = { ...publicHeaders };
publicHeaders["Authorization"] = "";
console.log(signed);
