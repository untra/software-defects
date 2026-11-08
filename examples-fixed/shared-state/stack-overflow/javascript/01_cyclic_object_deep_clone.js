function deepClone(value, seen = new Map()) {
    if (value === null || typeof value !== "object")
        return value;
    if (seen.has(value))
        return seen.get(value);
    const copy = {};
    seen.set(value, copy);
    for (const [k, v] of Object.entries(value))
        copy[k] = deepClone(v, seen);
    return copy;
}
const order = { id: 1, customer: { name: "ana" } };
order.customer.lastOrder = order;
try {
    const copy = deepClone(order);
    console.log("cycle preserved:", copy.customer.lastOrder === copy);
}
catch (e) {
    console.log(`${e.name}: ${e.message}`);
}
