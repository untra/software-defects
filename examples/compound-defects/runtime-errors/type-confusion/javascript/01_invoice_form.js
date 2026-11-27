const STARTING_CENTS = 1250;
const FORM_CENTS = "1425";
const DECIMAL_PLACES = 2;
class Invoice {
    cents = STARTING_CENTS;
    apply(form) {
        this.cents = form.cents;
    }
    display() {
        return this.cents.toFixed(DECIMAL_PLACES);
    }
}
function main() {
    const invoice = new Invoice();
    invoice.apply({ cents: FORM_CENTS });
    console.log("cents", invoice.display());
}
main();
