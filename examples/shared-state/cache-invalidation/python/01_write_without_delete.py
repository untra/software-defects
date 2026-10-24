prices = {"pen": 5}
cache = {}


def quote(sku):
    if sku not in cache:
        cache[sku] = prices[sku]
    return cache[sku]


def set_price(sku, price):
    prices[sku] = price


def main():
    print(quote("pen"))
    set_price("pen", 9)
    print(quote("pen"))


main()
