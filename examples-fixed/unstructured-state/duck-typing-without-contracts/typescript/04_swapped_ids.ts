function ship(userId: string, orderId: string): string {
  return `ship order ${orderId} to user ${userId}`;
}

const owner = "u-9";
const purchase = "o-4";
console.log(ship(owner, purchase));
