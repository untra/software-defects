const ITEMS = [{ sku: "pen" }, { sku: "pad" }];

function collect(items: { sku: string }[]): string[] {
  const labels: string[] = [];
  let i = 0;
  while (i <= items.length) {
    labels.push(items[i].sku);
    i++;
  }
  return labels;
}

function main(): void {
  console.log("labels", collect(ITEMS).join(","));
}

main();
