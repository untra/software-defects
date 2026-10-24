type Listed = {
  map(fn: (item: string) => string): string[];
};

function upperTitles(items: Listed): string {
  return items.map((item) => item.toUpperCase()).join(",");
}

const cell = "ada" as unknown as Listed;
console.log(upperTitles(cell));
