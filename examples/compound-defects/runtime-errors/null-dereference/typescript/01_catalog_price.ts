const CATALOG = [{ sku: "pen", cents: 150 }];
const REQUESTED_SKU = "pad";

function lookup(sku: string) {
  for (const item of CATALOG) {
    if (item.sku === sku) return item;
  }
}

function main(): void {
  const selected = lookup(REQUESTED_SKU);
  console.log("cents", selected.cents);
}

main();
