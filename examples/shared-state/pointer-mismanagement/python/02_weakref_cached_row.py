import weakref


class Row:
    def __init__(self, sku):
        self.sku = sku


def main():
    cache = weakref.WeakValueDictionary()
    row = Row("pen")
    cache["pen"] = row
    del row
    print(cache.get("pen"))


main()
