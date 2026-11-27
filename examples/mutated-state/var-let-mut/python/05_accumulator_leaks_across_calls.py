THRESHOLD = 50
SCALE = 2
running_total = 0


def filter_and_transform(items):
    global running_total
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
