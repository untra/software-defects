import random


def cache_key(sku):
    return f"{sku}-{random.random()}"


def main():
    random.seed(1)
    print(cache_key("pen"))
    print(cache_key("pen"))


main()
