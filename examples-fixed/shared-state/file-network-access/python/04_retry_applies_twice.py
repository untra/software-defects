def main():
    balance = 100

    processed = {}

    def charge(amount, operation_id):
        nonlocal balance
        if operation_id in processed:
            return processed[operation_id]
        balance -= amount
        processed[operation_id] = balance
        return balance

    def pay(amount):
        try:
            applied = charge(amount, "payment-1")
            raise RuntimeError("unknown result")
        except RuntimeError:
            charge(amount, "payment-1")
        return balance

    print(pay(40))


main()
