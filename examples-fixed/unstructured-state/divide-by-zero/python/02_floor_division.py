def pages(items, per_page):
    return None if per_page == 0 else items // per_page


def main():
    print(pages(10, 0))


main()
