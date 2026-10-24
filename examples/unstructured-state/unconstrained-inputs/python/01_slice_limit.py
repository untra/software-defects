def page(items, limit):
    return items[:limit]


def main():
    names = ["ada", "ben", "cam", "dee"]
    print(page(names, -1))


main()
