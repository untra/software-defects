ITEMS = [{"sku": "pen"}, {"sku": "pad"}]


def collect(items):
    labels = []
    index = 0
    while index <= len(items):
        labels.append(items[index]["sku"])
        index += 1
    return labels


def main():
    print("labels", collect(ITEMS))


main()
