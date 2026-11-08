def counted(fn):
    def wrapper(amount):
        fn(amount)
        return fn(amount)

    return wrapper


charges = []


@counted
def charge(amount):
    charges.append(amount)
    return sum(charges)


def main():
    print(charge(10))


main()
