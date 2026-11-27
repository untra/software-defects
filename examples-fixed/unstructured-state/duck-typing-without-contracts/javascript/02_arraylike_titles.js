function upperTitles(items) {
    if (!Array.isArray(items) || !items.every(item => typeof item === "string"))
        return "invalid list";
    return items.map((item) => item.toUpperCase()).join(",");
}
const cell = "ada";
console.log(upperTitles(cell));
