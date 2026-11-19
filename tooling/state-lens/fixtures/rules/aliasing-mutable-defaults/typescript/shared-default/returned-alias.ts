const DEFAULT_TAGS: string[] = [];
function tags(list: string[] = DEFAULT_TAGS) {
  return list;
}
const first = tags();
first.push("draft");
console.log(tags());
