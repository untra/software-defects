import sys

sys.setrecursionlimit(40)


def left(n):
    return right(n + 1)


def right(n):
    return left(n + 1)


def main():
    print(left(0))


main()
