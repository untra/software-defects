from datetime import date, timedelta


def main():
    start = date(2026, 1, 31)
    print(start + timedelta(days=30))


main()
