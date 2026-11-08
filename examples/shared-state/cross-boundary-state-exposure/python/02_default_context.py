def context(bag={}):
    return bag


def main():
    first = context()
    first["user"] = "ada"
    second = context()
    second["user"] = "ben"
    print(first["user"], second["user"])


main()
