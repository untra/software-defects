class Store:
    def save(self):
        return "store"


class Audit:
    def save(self):
        return "audit"


class Record(Audit, Store):
    pass


def main():
    print(Record().save())


main()
