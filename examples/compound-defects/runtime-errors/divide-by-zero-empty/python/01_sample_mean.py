SAMPLES = []


def mean(samples):
    total = 0
    for sample in samples:
        total += sample
    return total / len(samples)


def main():
    print("mean", mean(SAMPLES))


main()
