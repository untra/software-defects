def lookup(name):
    return "select id from people where name = ?", (name,)


def main():
    print(lookup("O'Brien"))


main()
