type Json = { [key: string]: unknown };

function deepClone(value: unknown, seen = new Map<object, unknown>()): unknown {
  if (value === null || typeof value !== "object") return value;
  if (seen.has(value)) return seen.get(value);
  const copy: Json = {};
  seen.set(value, copy);
  for (const [k, v] of Object.entries(value)) copy[k] = deepClone(v, seen);
  return copy;
}

const order: Json = { id: 1, customer: { name: "ana" } };
(order.customer as Json).lastOrder = order;

try {
  const copy = deepClone(order) as Json;
  console.log("cycle preserved:", (copy.customer as Json).lastOrder === copy);
} catch (e) {
  console.log(`${(e as Error).name}: ${(e as Error).message}`);
}
