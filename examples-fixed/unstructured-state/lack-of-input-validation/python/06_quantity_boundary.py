UNIT_PRICE = 10


def charge(quantity: int) -> int:
    if type(quantity) is not int or quantity < 0:
        raise ValueError("quantity rejected")
    return quantity * UNIT_PRICE


try:
    print("charge", charge(-3))
except ValueError as error:
    print(error)
