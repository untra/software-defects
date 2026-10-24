async function quote(sku: string): Promise<number> {
  if (sku === "bolt") {
    return 5;
  }
}

quote("nut").then((price) => {
  console.log("price", price * 2);
});
