const CENTS_PER_DOLLAR = 100;

type Charger = { amount(): number };

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

function charge(source: Charger): number {
  return source.amount();
}

console.log("charged", charge(card));
console.log("charged", charge(invoice));
