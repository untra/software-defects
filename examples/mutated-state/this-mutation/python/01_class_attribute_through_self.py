class Ledger:
    lines = []

    def post(self, amount):
        self.lines.append(amount)

    def balance(self):
        return sum(self.lines)


def main():
    ada = Ledger()
    ben = Ledger()
    ada.post(10)
    ben.post(4)
    print(ada.balance(), ben.balance())


main()
