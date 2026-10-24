def main():
    rows = ["keep", "drop", "drop", "keep"]
    for row in rows:
        if row == "drop":
            rows.remove(row)
    print(rows)


main()
