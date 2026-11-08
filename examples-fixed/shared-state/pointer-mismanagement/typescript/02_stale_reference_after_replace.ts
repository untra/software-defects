type Cart = { items: string[] };

const store: { cart: Cart } = { cart: { items: ["boots"] } };

function clearCart(): void {
  store.cart.items.length = 0;
}

const cartPanel = store.cart;
clearCart();
cartPanel.items.push("scarf");
console.log("panel shows:", cartPanel.items, "checkout sees:", store.cart.items);
