releases = [
    {"version": "1.2.0", "february": lambda year: 29 if year % 4 == 0 else 28},
    {"version": "1.4.0", "february": lambda _year: 28},
]


def resolve(catalog):
    return catalog[-1]


def main():
    picked = resolve(releases)
    print(picked["version"], picked["february"](2024))


main()
