class Roster:
    def __init__(self):
        self._names = ["ada"]

    def names(self):
        return self._names


def main():
    roster = Roster()
    seen = roster.names()
    seen.append("ben")
    print(roster.names())


main()
