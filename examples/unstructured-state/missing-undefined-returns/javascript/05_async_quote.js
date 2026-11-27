async function quote(sku) {
    if (sku === "bolt") {
        return 5;
    }
}
quote("nut").then((price) => {
    console.log("price", price * 2);
});
