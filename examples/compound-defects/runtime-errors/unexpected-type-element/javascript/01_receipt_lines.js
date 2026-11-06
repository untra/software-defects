const LINE_CENTS = 250;
const FORM_CENTS = "275";
const DECIMAL_PLACES = 2;
class Line {
    cents = LINE_CENTS;
    edit(cents) {
        this.cents = cents;
    }
}
function format(lines) {
    return lines.map((line) => line.cents.toFixed(DECIMAL_PLACES));
}
function main() {
    const line = new Line();
    const lines = [line];
    line.edit(FORM_CENTS);
    console.log("receipt", format(lines).join(","));
}
main();
