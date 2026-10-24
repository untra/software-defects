const OPENING_TOTAL = 0;
let total = OPENING_TOTAL;
let ledgerHeld = false;

function postInvoice(amount: number): void {
  if (ledgerHeld) throw new Error("ledger busy");
  ledgerHeld = true;
  try {
    const wholeAmount = amount | 0;
    total += wholeAmount;
  } finally {
    ledgerHeld = false;
  }
}

function main(): void {
  const INVOICE_AMOUNT = 19.75;
  postInvoice(INVOICE_AMOUNT);
  console.log("total", total);
}

main();
