def billing_february(year):
    if year % 4 == 0 and (year % 100 != 0 or year % 400 == 0):
        return 29
    return 28


def payroll_february(_year):
    return 28


def main():
    print("billing 2024 february", billing_february(2024))
    print("payroll 2024 february", payroll_february(2024))


main()
