const UNIT_PRICE = 10;

function charge(quantity) {
  return quantity * UNIT_PRICE;
}

console.log("charge", charge(-3));
