def lookup(name):
    return f"select id from people where name = '{name}'"


def main():
    print(lookup("O'Brien"))


main()
