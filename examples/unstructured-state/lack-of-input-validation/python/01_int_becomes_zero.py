def parse_qty(raw):
    try:
        return int(raw)
    except ValueError:
        return 0


def main():
    print(parse_qty("ten"))


main()
