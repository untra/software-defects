prices = {"pen": 5, "pencil": 2}
cache = dict(prices)


def set_price(sku, price):
    prices[sku] = price
    cache.pop("pencil", None)


def main():
    print(cache["pen"])
    set_price("pen", 9)
    print(cache.get("pen"), cache.get("pencil"))


main()
