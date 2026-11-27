def main():
    header = "content-length"
    name, separator, value = header.partition(":")
    print(value if separator else "invalid header")


main()
