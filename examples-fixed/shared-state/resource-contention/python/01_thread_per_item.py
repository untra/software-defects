import threading
from concurrent.futures import ThreadPoolExecutor

ITEMS = 8
MAX_WORKERS = 2


def main():
    peak = 0
    current = 0
    lock = threading.Lock()

    def work():
        nonlocal peak, current
        with lock:
            current += 1
            peak = max(peak, current)
        with lock:
            current -= 1

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        list(pool.map(lambda _: work(), range(ITEMS)))
    print(peak)


main()
