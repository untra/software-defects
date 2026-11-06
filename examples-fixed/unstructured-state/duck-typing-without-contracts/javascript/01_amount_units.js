const CENTS_PER_DOLLAR = 100;
const card = {
    cents: 250,
    amount() {
        return this.cents;
    },
};
const invoice = {
    dollars: 250,
    amount() {
        return this.dollars * CENTS_PER_DOLLAR;
    },
};
function charge(source) {
    if (typeof source?.amount !== "function") throw new TypeError("Expected an amount provider");
    const amount = source.amount();
    if (!Number.isFinite(amount)) throw new TypeError("Expected a finite amount");
    return amount;
}
console.log("charged", charge(card));
console.log("charged", charge(invoice));
