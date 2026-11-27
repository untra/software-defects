def selected(names, wanted):
    for name in names:
        if name == wanted:
            break
    return name


def main():
    print("selected", selected(["ada", "ben"], "cam"))


main()
