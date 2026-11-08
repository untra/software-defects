function normalizeQuantity(quantity: number): number {
  if (!quantity) {
    return 1;
  }
  return quantity;
}

console.log("qty", normalizeQuantity(0));
