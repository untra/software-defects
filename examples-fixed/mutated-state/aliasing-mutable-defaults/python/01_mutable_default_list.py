def add_tag(item, tags=None):
    if tags is None:
        tags = []
    tags.append(item)
    return tags


def main():
    print(add_tag("ada"))
    print(add_tag("ben"))


main()
