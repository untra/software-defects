import threading

OPENING_TOTAL = 0
INVOICE_AMOUNT = 19.75
ledger = threading.Lock()
total = OPENING_TOTAL


def post(amount):
    global total
    with ledger:
        whole_amount = int(amount)
        total += whole_amount


def main():
    post(INVOICE_AMOUNT)
    print("total", total)


main()
