const MAX_PAGE_ITEMS = 3;
const rows = ["ana", "ben", "cam", "dee"];
const limit = -1;
if (!Number.isInteger(limit) || limit < 0 || limit > MAX_PAGE_ITEMS) {
  console.log("page limit rejected");
} else {
  console.log(rows.slice(0, limit).join(","));
}
