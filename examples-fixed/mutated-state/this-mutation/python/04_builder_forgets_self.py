class Order:
    def __init__(self):
        self.sku = None

    def with_sku(self, sku):
        self.sku = sku
        return self


def main():
    order = Order().with_sku("pen")
    print(order.sku)


main()
