class Order:
    def __init__(self):
        self.sku = None

    def with_sku(self, sku):
        self.sku = sku


def main():
    order = Order().with_sku("pen")
    print(order.sku)


main()
