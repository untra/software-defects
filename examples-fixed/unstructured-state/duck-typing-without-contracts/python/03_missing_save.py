class Draft:
    def __init__(self, name):
        self.name = name


def store(record):
    save = getattr(record, "save", None)
    if not callable(save):
        return "record cannot be saved"
    return save()


def main():
    print(store(Draft("ada")))


main()
