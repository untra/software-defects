const INVOICE_CENTS = 450;
const receiptHelper = {
    receipt() {
        return String(this.amount());
    },
};
function main() {
    const billing = { totalCents: () => INVOICE_CENTS };
    const invoice = Object.assign(billing, receiptHelper);
    console.log("receipt", invoice.receipt());
}
main();
