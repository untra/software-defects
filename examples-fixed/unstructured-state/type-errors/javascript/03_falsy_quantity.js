function normalizeQuantity(quantity) {
    if (quantity === undefined) {
        return 1;
    }
    return quantity;
}
console.log("qty", normalizeQuantity(0));
