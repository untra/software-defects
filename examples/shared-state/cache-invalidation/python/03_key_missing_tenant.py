cache = {}


def quote(tenant, sku, price):
    if sku not in cache:
        cache[sku] = price
    return cache[sku]


def main():
    print(quote("ada", "pen", 5))
    print(quote("grace", "pen", 9))


main()
