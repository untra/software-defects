def stash(note, bag=[]):
    bag.append(note)
    return bag


def main():
    stash({"sku": "pen"})
    stash("ink")
    for note in stash({"sku": "pad"}):
        print(note["sku"])


main()
