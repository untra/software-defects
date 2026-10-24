INITIAL_RATE = 2
REVISED_RATE = 3
WEIGHT = 5
rate = INITIAL_RATE
quotes = {}


def shipping(weight):
    return weight * rate


def quote(weight):
    if weight not in quotes:
        quotes[weight] = shipping(weight)
    return quotes[weight]


def main():
    global rate
    quote(WEIGHT)
    rate = REVISED_RATE
    print("shipping", quote(WEIGHT))


main()
