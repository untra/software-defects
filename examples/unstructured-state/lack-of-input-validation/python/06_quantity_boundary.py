UNIT_PRICE = 10


def charge(quantity: int) -> int:
    return quantity * UNIT_PRICE


print("charge", charge(-3))
