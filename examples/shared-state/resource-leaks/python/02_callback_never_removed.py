listeners = []


def subscribe(fn):
    listeners.append(fn)


def handle(name):
    subscribe(lambda text: text)


def main():
    handle("ada")
    handle("ben")
    print(len(listeners))


main()
