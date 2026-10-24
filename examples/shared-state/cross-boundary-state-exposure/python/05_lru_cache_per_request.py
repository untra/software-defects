from functools import lru_cache

user = "ada"


@lru_cache
def quote(sku):
    return 5 if user == "ada" else 9


def main():
    global user
    print(quote("pen"))
    user = "grace"
    print(quote("pen"))


main()
