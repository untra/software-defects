import sys

sys.setrecursionlimit(40)


def send(attempt):
    return send(attempt + 1)


def main():
    print(send(1))


main()
