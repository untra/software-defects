const INITIAL_RATE = 2;
const REVISED_RATE = 3;
const WEIGHT = 5;
let rate = INITIAL_RATE;
const quotes = new Map();
function shipping(weight) {
    return weight * rate;
}
function quote(weight) {
    if (!quotes.has(weight))
        quotes.set(weight, shipping(weight));
    return quotes.get(weight);
}
function main() {
    quote(WEIGHT);
    rate = REVISED_RATE;
    console.log("shipping", quote(WEIGHT));
}
main();
