def discount(kind):
    if kind == "member":
        return 0.1
    return 0


def main():
    rate = discount("guest")
    print(1 - rate)


main()
