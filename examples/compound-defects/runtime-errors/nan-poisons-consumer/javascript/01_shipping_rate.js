const RATE_INPUT = "";
const RATE_LABELS = ["standard", "express", "priority"];
const STARTING_RATE = 1;
let rate = STARTING_RATE;
function quote(input) {
    rate = Number.parseFloat(input);
    return RATE_LABELS[Math.round(rate)].toUpperCase();
}
function main() {
    console.log("shipping", quote(RATE_INPUT));
}
main();
