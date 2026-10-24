import sys

sys.setrecursionlimit(40)


def walk(node):
    return 1 + walk(node["next"])


def main():
    node = {}
    node["next"] = node
    print(walk(node))


main()
