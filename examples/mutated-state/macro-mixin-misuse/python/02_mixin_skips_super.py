class Store:
    def save(self):
        return ["store"]


class Audit:
    def save(self):
        return ["audit"]


class Record(Audit, Store):
    def save(self):
        return Audit.save(self)


def main():
    print(Record().save())


main()
