const inventory = { widgets: 10 };
async function recordSale(quantity) {
    await new Promise((resolve) => setTimeout(resolve, quantity));
}
async function sell(quantity) {
    const current = inventory.widgets;
    await recordSale(quantity);
    inventory.widgets = current - quantity;
}
async function main() {
    await Promise.all([sell(3), sell(4)]);
    console.log("widgets left:", inventory.widgets);
}
main();
