class Account:
    def __init__(self):
        self.seed()

    def seed(self):
        self.bonus = 0


class Staff(Account):
    def seed(self):
        self.bonus = self.bonus + 5


def main():
    print(Staff().bonus)


main()
