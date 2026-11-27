const session = Object.freeze({ headers: { Authorization: "Bearer secret" } });
function normalize(headers) {
  return headers;
}
const publicHeaders = normalize(session.headers);
publicHeaders.Authorization = "";
console.log(session.headers.Authorization);
