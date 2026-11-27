class Prices:
    cache = {}

    def quote(self, sku, tenant):
        if sku not in self.cache:
            self.cache[sku] = 5 if tenant == "ada" else 9
        return self.cache[sku]


def main():
    prices = Prices()
    print(prices.quote("pen", "ada"), prices.quote("pen", "grace"))


main()
