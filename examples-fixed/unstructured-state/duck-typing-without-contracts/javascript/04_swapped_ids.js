function ship(userId, orderId) {
    return `ship order ${orderId} to user ${userId}`;
}
const owner = "u-9";
const purchase = "o-4";
console.log(ship(owner, purchase));
