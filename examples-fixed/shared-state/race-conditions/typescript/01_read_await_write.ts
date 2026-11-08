const inventory = { widgets: 10 };

async function recordSale(quantity: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, quantity));
}

async function sell(quantity: number): Promise<void> {
  if (quantity < 0 || inventory.widgets < quantity) throw new Error("insufficient inventory");
  inventory.widgets -= quantity;
  try { await recordSale(quantity); }
  catch (error) { inventory.widgets += quantity; throw error; }
}

async function main() {
  await Promise.all([sell(3), sell(4)]);
  console.log("widgets left:", inventory.widgets);
}

main();
