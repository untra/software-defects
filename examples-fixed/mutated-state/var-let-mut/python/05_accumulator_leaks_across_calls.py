THRESHOLD = 50
SCALE = 2


def filter_and_transform(items):
    running_total = 0
    result = []
    for item in items:
        running_total += item
        if running_total > THRESHOLD:
            result.append(item * SCALE)
    return result


def main():
    print(filter_and_transform([30, 25]))
    print(filter_and_transform([30, 25]))


main()
