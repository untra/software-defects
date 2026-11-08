class Cents:
    def __init__(self, value):
        self.value = value

    def amount(self):
        return self.value


class Dollars:
    def __init__(self, value):
        self.value = value

    def amount(self):
        return self.value


def charge(payment):
    return payment.amount()


def main():
    print(charge(Cents(250)))
    print(charge(Dollars(250)))


main()
