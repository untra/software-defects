rate = 0.05


def tax(amount, applied_rate=rate):
    return round(amount * applied_rate, 2)



def main():
    print("before", tax(100))
    tax(100, applied_rate=0.2)
    print("after", tax(100))


main()
