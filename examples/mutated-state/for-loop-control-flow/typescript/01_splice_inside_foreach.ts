const EMPTY = "empty";

const carts = [EMPTY, EMPTY, "full", EMPTY];
carts.forEach((cart, i) => {
  if (cart === EMPTY) carts.splice(i, 1);
});
console.log(carts);
