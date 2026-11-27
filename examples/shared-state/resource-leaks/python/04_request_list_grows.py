requests = []


def handle(name):
    requests.append(name)


def main():
    handle("ada")
    handle("ben")
    print(len(requests))


main()
