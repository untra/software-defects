const LINE_CENTS = 250;
const FORM_CENTS = "275";
const DECIMAL_PLACES = 2;

class Line {
  cents: any = LINE_CENTS;

  edit(cents: string): void {
    this.cents = cents;
  }
}

function format(lines: Line[]): string[] {
  return lines.map((line) => line.cents.toFixed(DECIMAL_PLACES));
}

function main(): void {
  const line = new Line();
  const lines = [line];
  line.edit(FORM_CENTS);
  console.log("receipt", format(lines).join(","));
}

main();
