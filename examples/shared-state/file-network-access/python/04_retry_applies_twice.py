def main():
    balance = 100

    def charge(amount):
        nonlocal balance
        balance -= amount
        return balance

    def pay(amount):
        try:
            applied = charge(amount)
            raise RuntimeError("unknown result")
        except RuntimeError:
            charge(amount)
        return balance

    print(pay(40))


main()
