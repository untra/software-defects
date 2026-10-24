def main():
    rows = ["keep", "drop", "drop", "keep"]
    index = 0
    while index < len(rows):
        if rows[index] == "drop":
            del rows[index]
        index += 1
    print(rows)


main()
