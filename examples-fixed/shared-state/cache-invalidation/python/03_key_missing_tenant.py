cache = {}


def quote(tenant, sku, price):
    key = (tenant, sku, price)
    if key not in cache:
        cache[key] = price
    return cache[key]


def main():
    print(quote("ada", "pen", 5))
    print(quote("grace", "pen", 9))


main()
