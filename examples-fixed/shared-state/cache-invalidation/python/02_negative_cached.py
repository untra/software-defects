cache = {}
stored = set()


def lookup(sku):
    if sku in cache:
        return cache[sku]
    if sku not in stored:
        return None
    return sku


def main():
    print(lookup("pen"))
    stored.add("pen")
    print(lookup("pen"))


main()
