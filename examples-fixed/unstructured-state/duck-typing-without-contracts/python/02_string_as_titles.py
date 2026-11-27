def titles(rows):
    if not isinstance(rows, list) or not all(isinstance(row, str) for row in rows):
        return "invalid titles"
    return [row for row in rows]


def main():
    print(titles("ADA"))


main()
