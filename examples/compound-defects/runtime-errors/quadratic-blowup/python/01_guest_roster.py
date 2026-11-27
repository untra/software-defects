GUESTS = ["Ari", "Bo", "Cy", "Dee"]


def roster(names):
    accepted = []
    comparisons = 0
    for name in names:
        present = False
        for earlier in accepted:
            comparisons += 1
            if earlier == name:
                present = True
        if not present:
            accepted.append(name)
    return comparisons


def main():
    print("comparisons", roster(GUESTS))


main()
