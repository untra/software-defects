const PRICES_CENTS = { coffee: 10, bagel: 20 };
const AMOUNT_TENDERED = 30;

const total = PRICES_CENTS.coffee + PRICES_CENTS.bagel;
console.log(total === AMOUNT_TENDERED ? "paid in full" : `short by ${AMOUNT_TENDERED - total}`);
