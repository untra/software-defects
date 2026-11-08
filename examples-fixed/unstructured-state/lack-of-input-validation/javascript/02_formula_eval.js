const formula = "2 + 2, 100";
const parts = /^([0-9]+)\s*\+\s*([0-9]+)$/.exec(formula);
const total = parts ? Number(parts[1]) + Number(parts[2]) : "formula rejected";
console.log("total", total);
