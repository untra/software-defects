const PREVIEW_LIMIT = 1;

class Inventory {
  private readonly skus: string[] = [];
  add(sku: string) {
    this.skus.push(sku);
  }
  list(): string[] {
    return this.skus;
  }
}

const inventory = new Inventory();
inventory.add("sku-1");
inventory.add("sku-2");
inventory.add("sku-3");
const preview = inventory.list();
preview.splice(PREVIEW_LIMIT);
console.log("preview:", preview);
console.log("inventory:", inventory.list());
