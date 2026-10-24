class Handle:
    open_count = 0

    def __init__(self):
        Handle.open_count += 1
        self.closed = False

    def write(self, text):
        raise OSError("disk full")

    def close(self):
        self.closed = True
        Handle.open_count -= 1


def main():
    handle = Handle()
    try:
        handle.write("ada")
        handle.close()
    except OSError:
        pass
    print(Handle.open_count)


main()
