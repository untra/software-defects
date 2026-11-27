current_user = None


def render():
    return f"hello {current_user}"


def main():
    global current_user
    current_user = "ada"
    pending = render
    current_user = "ben"
    print(pending())


main()
