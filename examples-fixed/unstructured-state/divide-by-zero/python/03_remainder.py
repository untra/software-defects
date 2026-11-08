def leftover(items, per_box):
    return None if per_box == 0 else items % per_box


def main():
    print(leftover(10, 0))


main()
