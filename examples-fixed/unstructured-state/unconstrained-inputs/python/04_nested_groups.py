MAX_DEPTH = 3


def groups(depth):
    if type(depth) is not int or not 0 <= depth <= MAX_DEPTH:
        raise ValueError("group depth exceeded")
    node = []
    for _ in range(depth):
        node = [node]
    height = 0
    while isinstance(node, list) and node:
        height += 1
        node = node[0]
    return height


def main():
    try:
        print(groups(5))
    except ValueError as error:
        print(error)


main()
