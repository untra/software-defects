def main():
    shown = []
    for cents in (0.5, 1.5, 2.5):
        shown.append(str(round(cents)))
    print(" ".join(shown))


main()
