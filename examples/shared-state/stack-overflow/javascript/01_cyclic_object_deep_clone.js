function deepClone(value) {
    if (value === null || typeof value !== "object")
        return value;
    const copy = {};
    for (const [k, v] of Object.entries(value))
        copy[k] = deepClone(v);
    return copy;
}
const order = { id: 1, customer: { name: "ana" } };
order.customer.lastOrder = order;
try {
    deepClone(order);
}
catch (e) {
    console.log(`${e.name}: ${e.message}`);
}
