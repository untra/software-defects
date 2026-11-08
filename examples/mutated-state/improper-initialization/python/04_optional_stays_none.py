class Quote:
    def __init__(self):
        self.extra = None

    def total(self, base):
        return base + self.extra


def main():
    print(Quote().total(10))


main()
