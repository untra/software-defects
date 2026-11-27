const OPENING_TOTAL = 100;
let total = OPENING_TOTAL;
let ledgerHeld = false;

function postQuote(amount: number, units: number): void {
  if (ledgerHeld) throw new Error("ledger busy");
  ledgerHeld = true;
  try {
    const quote = amount / units;
    total += quote;
  } finally {
    ledgerHeld = false;
  }
}

function main(): void {
  const EMPTY_AMOUNT = 0;
  const EMPTY_UNITS = 0;
  postQuote(EMPTY_AMOUNT, EMPTY_UNITS);
  console.log("total", total);
}

main();
