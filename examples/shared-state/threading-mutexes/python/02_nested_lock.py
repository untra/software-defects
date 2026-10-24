import threading


def main():
    lock = threading.Lock()
    lock.acquire()
    entered = lock.acquire(timeout=0.05)
    print("blocked" if not entered else "entered", entered)
    lock.release()


main()
