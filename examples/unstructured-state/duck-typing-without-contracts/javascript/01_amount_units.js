const card = {
    cents: 250,
    amount() {
        return this.cents;
    },
};
const invoice = {
    dollars: 250,
    amount() {
        return this.dollars;
    },
};
function charge(source) {
    return source.amount();
}
console.log("charged", charge(card));
console.log("charged", charge(invoice));
