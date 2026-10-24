class Handle:
    def __init__(self):
        self.closed = False

    def write(self, text):
        return len(text)

    def close(self):
        raise OSError("close failed")
        self.closed = True


def main():
    handle = Handle()
    try:
        handle.write("ada")
        handle.close()
    except OSError:
        pass
    print(handle.closed)


main()
