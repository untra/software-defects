const INITIAL_QUANTITY = 2;
const QUANTITY_PLACES = 0;
const PAYLOAD: { qty?: number } = {};
const cart: { qty?: number } = { qty: INITIAL_QUANTITY };

function apply(payload: { qty?: number }): void {
  cart.qty = payload.qty;
}

function main(): void {
  apply(PAYLOAD);
  console.log("quantity", cart.qty.toFixed(QUANTITY_PLACES));
}

main();
