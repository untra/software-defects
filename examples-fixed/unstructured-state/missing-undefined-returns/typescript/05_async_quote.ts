async function quote(sku: string): Promise<number | null> {
  if (sku === "bolt") {
    return 5;
  }
  return null;
}

quote("nut").then((price) => {
  console.log("price", price === null ? "unavailable" : price * 2);
});
