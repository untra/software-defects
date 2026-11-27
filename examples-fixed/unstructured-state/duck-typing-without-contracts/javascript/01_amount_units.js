const CENTS_PER_DOLLAR = 100;
const card = {
    cents: 250,
    amountCents() {
        return this.cents;
    },
};
const invoice = {
    dollars: 250,
    amountCents() {
        return this.dollars * CENTS_PER_DOLLAR;
    },
};
function charge(source) {
    if (typeof source?.amountCents !== "function") throw new TypeError("Expected an amountCents provider");
    const amountCents = source.amountCents();
    if (!Number.isFinite(amountCents)) throw new TypeError("Expected a finite amountCents");
    return amountCents;
}
console.log("charged", charge(card));
console.log("charged", charge(invoice));
