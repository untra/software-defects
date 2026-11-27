type Note = { sku: string };
const DEFAULT_BAG: Array<Note | string> = [];
const FIRST_NOTE = { sku: "pen" };
const MIDDLE_NOTE = "ink";
const LAST_NOTE = { sku: "pad" };

function stash(note: Note | string, bag = DEFAULT_BAG) {
  bag.push(note);
  return bag;
}

function main(): void {
  stash(FIRST_NOTE);
  stash(MIDDLE_NOTE);
  const labels = stash(LAST_NOTE).map((note) => note.sku.toUpperCase());
  console.log("labels", labels.join(","));
}

main();
