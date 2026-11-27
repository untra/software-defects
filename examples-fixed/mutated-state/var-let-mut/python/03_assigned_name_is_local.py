subtotal = 40


def apply_tax(rate):
    return subtotal + round(subtotal * rate)


def main():
    print(apply_tax(0.1))


main()
