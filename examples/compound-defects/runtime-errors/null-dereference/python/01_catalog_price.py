CATALOG = [{"sku": "pen", "cents": 150}]
REQUESTED_SKU = "pad"


def lookup(sku):
    for item in CATALOG:
        if item["sku"] == sku:
            return item


def main():
    selected = lookup(REQUESTED_SKU)
    print("cents", selected["cents"])


main()
