const request = { headers: {} as Record<string, string> };
function withHeader(target: { headers: Record<string, string> }) {
  return target;
}
withHeader(request).headers["Accept"] = "application/json";
console.log(request.headers);
