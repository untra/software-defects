import threading
import time

WORKERS = 8
PER_WORKER = 400


def main():
    total = 0

    def add():
        nonlocal total
        for _ in range(PER_WORKER):
            current = total
            time.sleep(0)
            total = current + 1

    threads = [threading.Thread(target=add) for _ in range(WORKERS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    print(f"recorded {total} of {WORKERS * PER_WORKER}")


main()
