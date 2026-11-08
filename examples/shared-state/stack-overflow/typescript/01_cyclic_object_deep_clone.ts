type Json = { [key: string]: unknown };

function deepClone(value: unknown): unknown {
  if (value === null || typeof value !== "object") return value;
  const copy: Json = {};
  for (const [k, v] of Object.entries(value)) copy[k] = deepClone(v);
  return copy;
}

const order: Json = { id: 1, customer: { name: "ana" } };
(order.customer as Json).lastOrder = order;

try {
  deepClone(order);
} catch (e) {
  console.log(`${(e as Error).name}: ${(e as Error).message}`);
}
