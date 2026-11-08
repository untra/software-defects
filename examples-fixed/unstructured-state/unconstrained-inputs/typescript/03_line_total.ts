const price = 1e308;
const qty = 10;
console.log("total", Number.isFinite(price * qty) ? price * qty : "rejected");
