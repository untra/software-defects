def days_v1(year):
    return 29 if year % 4 == 0 else 28


def days_v2(_year):
    return 28


def main():
    print("invoice", days_v1(2024))
    print("payroll", days_v2(2024))


main()
