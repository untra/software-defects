tax_rate = 0.1


def total(amount):
    return amount + amount * tax_rate


def main():
    global tax_rate
    print(total(100))
    tax_rate = 0.2
    print(total(100))


main()
