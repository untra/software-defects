const signed: Readonly<Record<string, string>> = { Authorization: "Bearer secret" };
function normalize(headers: Record<string, string>) {
  return headers;
}
const publicHeaders = normalize(signed);
publicHeaders["Authorization"] = "";
console.log(signed);
