import threading


def main():
    lock = threading.RLock()
    with lock:
        entered = lock.acquire(timeout=0.05)
        try:
            print("blocked" if not entered else "entered", entered)
        finally:
            if entered:
                lock.release()


main()
