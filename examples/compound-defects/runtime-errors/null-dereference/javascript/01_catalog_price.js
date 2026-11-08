const CATALOG = [{ sku: "pen", cents: 150 }];
const REQUESTED_SKU = "pad";
function lookup(sku) {
    for (const item of CATALOG) {
        if (item.sku === sku)
            return item;
    }
}
function main() {
    const selected = lookup(REQUESTED_SKU);
    console.log("cents", selected.cents);
}
main();
