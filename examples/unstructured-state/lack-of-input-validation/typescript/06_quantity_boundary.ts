const UNIT_PRICE = 10;

function charge(quantity: number): number {
  return quantity * UNIT_PRICE;
}

console.log("charge", charge(-3));
