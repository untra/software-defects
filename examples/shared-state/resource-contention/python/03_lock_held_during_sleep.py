import threading


def main():
    lock = threading.Lock()
    holding = threading.Event()
    release = threading.Event()

    def holder():
        lock.acquire()
        holding.set()
        release.wait()
        lock.release()

    thread = threading.Thread(target=holder)
    thread.start()
    holding.wait()
    entered = lock.acquire(timeout=0.05)
    print("blocked" if not entered else "entered", entered)
    release.set()
    thread.join()


main()
