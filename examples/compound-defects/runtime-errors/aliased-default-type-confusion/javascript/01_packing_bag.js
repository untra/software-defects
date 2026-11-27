const DEFAULT_BAG = [];
const FIRST_NOTE = { sku: "pen" };
const MIDDLE_NOTE = "ink";
const LAST_NOTE = { sku: "pad" };
function stash(note, bag = DEFAULT_BAG) {
    bag.push(note);
    return bag;
}
function main() {
    stash(FIRST_NOTE);
    stash(MIDDLE_NOTE);
    const labels = stash(LAST_NOTE).map((note) => note.sku.toUpperCase());
    console.log("labels", labels.join(","));
}
main();
