def selected(names, wanted):
    for name in names:
        if name == wanted:
            return name
    return f"{wanted} is missing"


def main():
    print("selected", selected(["ada", "ben"], "cam"))


main()
