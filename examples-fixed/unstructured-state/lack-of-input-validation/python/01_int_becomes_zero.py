def parse_qty(raw):
    return int(raw)


def main():
    try:
        print(parse_qty("ten"))
    except ValueError:
        print("quantity rejected")


main()
