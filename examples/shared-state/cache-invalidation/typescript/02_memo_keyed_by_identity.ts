const totals = new WeakMap<number[], number>();

function cartTotal(prices: number[]): number {
  const cached = totals.get(prices);
  if (cached !== undefined) return cached;
  const total = prices.reduce((sum, p) => sum + p, 0);
  totals.set(prices, total);
  return total;
}

const cart = [10, 20];
cartTotal(cart);
cart.push(30);
console.log("total charged:", cartTotal(cart));
