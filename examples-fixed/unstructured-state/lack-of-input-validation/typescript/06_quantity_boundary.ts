const UNIT_PRICE = 10;

function charge(quantity: number): number {
  if (!Number.isSafeInteger(quantity) || quantity < 0) {
    throw new Error("quantity rejected");
  }
  return quantity * UNIT_PRICE;
}

try {
  console.log("charge", charge(-3));
} catch (error) {
  console.log(error instanceof Error ? error.message : "quantity rejected");
}
