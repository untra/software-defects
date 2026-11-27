def charge(qty, price):
    if qty < 0 or price < 0:
        raise ValueError("quantity or price rejected")
    return qty * price


def main():
    try:
        print(charge(-3, 10))
    except ValueError as error:
        print(error)


main()
