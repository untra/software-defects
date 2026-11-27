def main():
    raw = "10px"
    print(int(raw) if raw.isdecimal() else "invalid quantity")


main()
