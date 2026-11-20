def total(base, formula):
    if formula != "base":
        raise ValueError("formula rejected")
    return base


def main():
    try:
        print(total(100, "base + adjust"))
    except ValueError as error:
        print(error)


main()
