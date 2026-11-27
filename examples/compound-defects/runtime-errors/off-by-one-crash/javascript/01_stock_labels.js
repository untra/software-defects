const ITEMS = [{ sku: "pen" }, { sku: "pad" }];
function collect(items) {
    const labels = [];
    let i = 0;
    while (i <= items.length) {
        labels.push(items[i].sku);
        i++;
    }
    return labels;
}
function main() {
    console.log("labels", collect(ITEMS).join(","));
}
main();
