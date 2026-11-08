const INVOICE_CENTS = 450;

type Billing = { totalCents(): number; amount?: () => number };

const receiptHelper = {
  receipt(this: Billing): string {
    return String(this.amount());
  },
};

function main(): void {
  const billing: Billing = { totalCents: () => INVOICE_CENTS };
  const invoice = Object.assign(billing, receiptHelper);
  console.log("receipt", invoice.receipt());
}

main();
