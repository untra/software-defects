import threading

OPENING_TOTAL = 100.0
EMPTY_AMOUNT = 0.0
EMPTY_UNITS = 0
ledger = threading.Lock()
total = OPENING_TOTAL


def quote(amount, units):
    try:
        return amount / units
    except ZeroDivisionError:
        return float("nan")


def post(amount):
    global total
    with ledger:
        total += amount


def main():
    post(quote(EMPTY_AMOUNT, EMPTY_UNITS))
    print("total", total)


main()
