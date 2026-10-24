const EMPTY = "empty";

const carts = [EMPTY, EMPTY, "full", EMPTY];
for (let i = carts.length - 1; i >= 0; i--) {
  if (carts[i] === EMPTY) carts.splice(i, 1);
}
console.log(carts);
