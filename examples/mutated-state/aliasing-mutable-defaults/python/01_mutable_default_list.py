def add_tag(item, tags=[]):
    tags.append(item)
    return tags


def main():
    print(add_tag("ada"))
    print(add_tag("ben"))


main()
