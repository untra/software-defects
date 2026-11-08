function normalizeQuantity(quantity: number | undefined): number {
  if (quantity === undefined) {
    return 1;
  }
  return quantity;
}

console.log("qty", normalizeQuantity(0));
