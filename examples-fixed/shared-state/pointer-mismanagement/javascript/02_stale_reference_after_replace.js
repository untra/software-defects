const store = { cart: { items: ["boots"] } };
function clearCart() {
    store.cart.items.length = 0;
}
const cartPanel = store.cart;
clearCart();
cartPanel.items.push("scarf");
console.log("panel shows:", cartPanel.items, "checkout sees:", store.cart.items);
