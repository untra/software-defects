function upperTitles(items) {
    return items.map((item) => item.toUpperCase()).join(",");
}
const cell = "ada";
console.log(upperTitles(cell));
