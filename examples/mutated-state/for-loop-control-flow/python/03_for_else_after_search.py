def main():
    names = ["ada", "ben", "cam"]
    found = None
    for name in names:
        if name == "ben":
            found = name
    else:
        found = "missing"
    print(found)


main()
