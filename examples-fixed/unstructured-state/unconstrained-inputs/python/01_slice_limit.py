MAX_PAGE_ITEMS = 3


def page(items, limit):
    if type(limit) is not int or not 0 <= limit <= MAX_PAGE_ITEMS:
        raise ValueError("page limit rejected")
    return items[:limit]


def main():
    names = ["ada", "ben", "cam", "dee"]
    try:
        print(page(names, -1))
    except ValueError as error:
        print(error)


main()
