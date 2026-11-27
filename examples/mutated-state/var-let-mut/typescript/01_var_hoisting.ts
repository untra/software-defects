const DISCOUNT_CODE = "SAVE10";
const DISCOUNT_RATE = 0.1;

function applyDiscount(price: number, code: string): number {
  if (code === DISCOUNT_CODE) {
    var rate = DISCOUNT_RATE;
  }
  return price - price * rate;
}

console.log("with code:", applyDiscount(100, "SAVE10"));
console.log("without code:", applyDiscount(100, "NONE"));
