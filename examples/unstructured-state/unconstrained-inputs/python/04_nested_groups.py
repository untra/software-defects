def groups(depth):
    node = []
    for _ in range(depth):
        node = [node]
    height = 0
    while isinstance(node, list) and node:
        height += 1
        node = node[0]
    return height


def main():
    print(groups(5))


main()
