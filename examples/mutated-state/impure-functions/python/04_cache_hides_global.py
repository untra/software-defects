from functools import cache

rate = 0.1


@cache
def taxed(amount):
    return amount + amount * rate


def main():
    global rate
    print(taxed(100))
    rate = 0.2
    print(taxed(100))


main()
