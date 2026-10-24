class Account:
    def __init__(self, kind):
        if kind == "staff":
            self.bonus = 5


def main():
    print(Account("guest").bonus)


main()
