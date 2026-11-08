import threading


def main():
    lock = threading.Lock()

    def work():
        lock.acquire()
        raise RuntimeError("disk full")
        lock.release()

    try:
        work()
    except RuntimeError:
        pass
    entered = lock.acquire(timeout=0.05)
    print("blocked" if not entered else "entered", entered)


main()
