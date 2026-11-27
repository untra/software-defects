const TAX_PERCENT = 10;
const PERCENT = 100;
const COUPON = 10;

class Invoice {
  private base: number;
  private taxed = false;
  private discounted = false;
  get subtotal() {
    const net = this.base - (this.discounted ? COUPON : 0);
    return net + (this.taxed ? net * TAX_PERCENT / PERCENT : 0);
  }
  constructor(subtotal: number) {
    this.base = subtotal;
  }
  applyTax() {
    this.taxed = true;
    return this.subtotal;
  }
  applyCoupon() {
    this.discounted = true;
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
