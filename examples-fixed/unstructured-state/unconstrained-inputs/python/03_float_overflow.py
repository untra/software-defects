import math


def main():
    total = 1e308 * 10
    print(total if math.isfinite(total) else "total rejected")


main()
