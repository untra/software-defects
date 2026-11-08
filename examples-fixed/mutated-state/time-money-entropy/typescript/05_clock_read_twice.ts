const HOLD_MS = 1000;
const VALIDATION_COST_MS = 5;

function validateCart() {
  const until = Date.now() + VALIDATION_COST_MS;
  while (Date.now() < until);
}

function reserveInventory() {
  const createdAt = Date.now();
  validateCart();
  const expiresAt = createdAt + HOLD_MS;
  return { createdAt, expiresAt };
}

const hold = reserveInventory();
console.log("hold window ms:", hold.expiresAt - hold.createdAt);
