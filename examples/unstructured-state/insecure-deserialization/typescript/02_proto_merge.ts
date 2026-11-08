const incoming = JSON.parse('{"name":"ada","__proto__":{"role":"admin"}}');
const record: Record<string, unknown> = { name: "" };
for (const key of Object.keys(incoming)) {
  record[key] = incoming[key];
}
console.log("role", record["role"], "own", Object.hasOwn(record, "role"));
