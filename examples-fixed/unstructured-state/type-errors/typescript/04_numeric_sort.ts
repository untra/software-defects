const labels = ["10", "2", "1"];
labels.sort((a, b) => Number(a) - Number(b));
console.log("order", labels.join(","));
