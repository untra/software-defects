const INITIAL_QUANTITY = 2;
const QUANTITY_PLACES = 0;
const PAYLOAD = {};
const cart = { qty: INITIAL_QUANTITY };
function apply(payload) {
    cart.qty = payload.qty;
}
function main() {
    apply(PAYLOAD);
    console.log("quantity", cart.qty.toFixed(QUANTITY_PLACES));
}
main();
