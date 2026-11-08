const totals = new Map();
function cartTotal(prices) {
    const key = JSON.stringify(prices);
    const cached = totals.get(key);
    if (cached !== undefined)
        return cached;
    const total = prices.reduce((sum, p) => sum + p, 0);
    totals.set(key, total);
    return total;
}
const cart = [10, 20];
cartTotal(cart);
cart.push(30);
console.log("total charged:", cartTotal(cart));
