const rows = ["ana", "ben", "cam", "dee"];
const limit = -1;
const page = Number.isInteger(limit) && limit >= 0 ? rows.slice(0, limit) : rows;
console.log(page.join(","));
