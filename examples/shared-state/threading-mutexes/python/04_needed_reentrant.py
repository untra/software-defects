import threading


def main():
    lock = threading.Lock()

    def inner():
        got = lock.acquire(timeout=0.05)
        if got:
            lock.release()
        return got

    def outer():
        lock.acquire()
        try:
            return inner()
        finally:
            lock.release()

    got = outer()
    print("blocked" if not got else "entered", got)


main()
