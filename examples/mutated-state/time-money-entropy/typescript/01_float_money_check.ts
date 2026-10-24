const PRICES_USD = { coffee: 0.1, bagel: 0.2 };
const AMOUNT_TENDERED = 0.3;

const total = PRICES_USD.coffee + PRICES_USD.bagel;
console.log(total === AMOUNT_TENDERED ? "paid in full" : `short by ${AMOUNT_TENDERED - total}`);
