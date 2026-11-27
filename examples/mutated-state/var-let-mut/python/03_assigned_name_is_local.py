subtotal = 40


def apply_tax(rate):
    if rate:
        subtotal = subtotal + round(subtotal * rate)
    return subtotal


def main():
    print(apply_tax(0.1))


main()
