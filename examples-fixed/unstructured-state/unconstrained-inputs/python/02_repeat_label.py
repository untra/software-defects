MAX_LABEL_LENGTH = 32


def label(requested):
    if type(requested) is not int or not 0 <= requested <= MAX_LABEL_LENGTH:
        raise ValueError("label rejected")
    return "x" * requested


def main():
    try:
        print(len(label(40)))
    except ValueError as error:
        print(error)


main()
