const TAX_PERCENT = 10;
const PERCENT = 100;
const COUPON = 10;

class Invoice {
  subtotal: number;
  constructor(subtotal: number) {
    this.subtotal = subtotal;
  }
  applyTax() {
    this.subtotal += (this.subtotal * TAX_PERCENT) / PERCENT;
    return this.subtotal;
  }
  applyCoupon() {
    this.subtotal -= COUPON;
    return this.subtotal;
  }
}

const a = new Invoice(100);
a.applyTax();
a.applyCoupon();
const b = new Invoice(100);
b.applyCoupon();
b.applyTax();
const retried = new Invoice(100);
retried.applyTax();
retried.applyTax();
console.log("tax then coupon:", a.subtotal);
console.log("coupon then tax:", b.subtotal);
console.log("tax retried:", retried.subtotal);
