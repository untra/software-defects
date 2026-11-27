import { EventEmitter } from "node:events";

const TAX_RATE = 0.1;

const cart = new EventEmitter();
const state = { subtotal: 100, total: 0 };

cart.on("subtotal-changed", () => {
  state.total = state.subtotal * (1 + TAX_RATE);
  cart.emit("total-changed");
});

cart.on("total-changed", () => {
  state.subtotal = Math.round(state.total);
  cart.emit("subtotal-changed");
});

try {
  cart.emit("subtotal-changed");
} catch (e) {
  console.log(`${(e as Error).name}: ${(e as Error).message}`);
}
