CENTS_PER_DOLLAR = 100

class Cents:
    def __init__(self, value):
        self.value = value

    def amount_cents(self):
        return self.value


class Dollars:
    def __init__(self, value):
        self.value = value

    def amount_cents(self):
        return self.value * CENTS_PER_DOLLAR


def charge(payment):
    return payment.amount_cents()


def main():
    print(charge(Cents(250)))
    print(charge(Dollars(250)))


main()
