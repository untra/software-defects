import random


def main():
    random.seed(1)
    first = "control" if random.random() < 0.5 else "trial"
    second = "control" if random.random() < 0.5 else "trial"
    print(first, second)


main()
