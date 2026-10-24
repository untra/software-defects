class Draft:
    def __init__(self, name):
        self.name = name


def store(record):
    record.save()


def main():
    store(Draft("ada"))


main()
