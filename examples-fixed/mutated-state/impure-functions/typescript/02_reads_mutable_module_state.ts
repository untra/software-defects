const BASE_PRICE = 100;
const PROMO_PERCENT_OFF = 20;
const PERCENT = 100;

let promoActive = false;

function quote(quantity: number, promotion: boolean): number {
  const unit = promotion ? BASE_PRICE * (1 - PROMO_PERCENT_OFF / PERCENT) : BASE_PRICE;
  return unit * quantity;
}

function enableBlackFriday() {
  promoActive = true;
}

const cartPromotion = promoActive;
const cartQuote = quote(2, cartPromotion);
enableBlackFriday();
const checkoutCharge = quote(2, cartPromotion);
console.log("quoted", cartQuote, "charged", checkoutCharge);
