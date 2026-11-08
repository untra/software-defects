type Charger = { amount(): number };

const card: Charger = {
  cents: 250,
  amount() {
    return this.cents;
  },
};

const invoice: Charger = {
  dollars: 250,
  amount() {
    return this.dollars;
  },
};

function charge(source: Charger): number {
  return source.amount();
}

console.log("charged", charge(card));
console.log("charged", charge(invoice));
