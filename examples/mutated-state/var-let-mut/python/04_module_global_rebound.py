rate = 0.05


def tax(amount):
    return round(amount * rate, 2)


def set_rate(next_rate):
    global rate
    rate = next_rate


def main():
    print("before", tax(100))
    set_rate(0.2)
    print("after", tax(100))


main()
