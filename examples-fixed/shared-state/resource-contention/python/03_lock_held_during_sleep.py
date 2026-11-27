import threading


def main():
    lock = threading.Lock()
    holding = threading.Event()
    release = threading.Event()

    def holder():
        with lock:
            holding.set()
        release.wait()

    thread = threading.Thread(target=holder)
    thread.start()
    holding.wait()
    entered = lock.acquire(timeout=0.05)
    print("blocked" if not entered else "entered", entered)
    if entered:
        lock.release()
    release.set()
    thread.join()


main()
