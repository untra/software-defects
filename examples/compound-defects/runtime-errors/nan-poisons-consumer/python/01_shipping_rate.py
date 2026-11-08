RATE_INPUT = "NaN"
RATE_LABELS = ["standard", "express", "priority"]
STARTING_RATE = 1.0
rate = STARTING_RATE


def quote(raw):
    global rate
    rate = float(raw)
    return RATE_LABELS[round(rate)].upper()


def main():
    print("shipping", quote(RATE_INPUT))


main()
