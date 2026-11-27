async function quote(sku) {
    if (sku === "bolt") {
        return 5;
    }
    return null;
}
quote("nut").then((price) => {
    console.log("price", price === null ? "unavailable" : price * 2);
});
