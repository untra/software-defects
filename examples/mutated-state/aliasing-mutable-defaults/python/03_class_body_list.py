class Cart:
    items = []

    def add(self, sku):
        self.items.append(sku)


def main():
    morning = Cart()
    afternoon = Cart()
    morning.add("pen")
    print(afternoon.items)


main()
