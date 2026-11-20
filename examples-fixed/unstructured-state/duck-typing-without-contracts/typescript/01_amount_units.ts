const CENTS_PER_DOLLAR = 100;

type Charger = { amountCents(): number };

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

function charge(source: Charger): number {
  return source.amountCents();
}

console.log("charged", charge(card));
console.log("charged", charge(invoice));
