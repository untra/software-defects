const STARTING_CENTS = 1250;
const FORM_CENTS = "1425";
const DECIMAL_PLACES = 2;

class Invoice {
  cents: any = STARTING_CENTS;

  apply(form: { cents: string }): void {
    this.cents = form.cents;
  }

  display(): string {
    return this.cents.toFixed(DECIMAL_PLACES);
  }
}

function main(): void {
  const invoice = new Invoice();
  invoice.apply({ cents: FORM_CENTS });
  console.log("cents", invoice.display());
}

main();
