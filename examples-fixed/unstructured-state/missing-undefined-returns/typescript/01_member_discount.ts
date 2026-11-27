function discount(isMember: boolean): number {
  if (isMember) {
    return 0.1;
  }
  return 0;
}

const guest = false;
const base = 100;
const price = base * (1 - discount(guest));
console.log("price", price);
