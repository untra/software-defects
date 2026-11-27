import threading


def main():
    lock = threading.Lock()

    def work():
        with lock:
            raise RuntimeError("disk full")

    try:
        work()
    except RuntimeError:
        pass
    entered = lock.acquire(timeout=0.05)
    try:
        print("blocked" if not entered else "entered", entered)
    finally:
        if entered:
            lock.release()


main()
