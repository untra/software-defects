import threading

ITEMS = 8


def main():
    peak = 0
    current = 0
    lock = threading.Lock()
    barrier = threading.Barrier(ITEMS)

    def work():
        nonlocal peak, current
        with lock:
            current += 1
            peak = max(peak, current)
        barrier.wait()
        with lock:
            current -= 1

    threads = [threading.Thread(target=work) for _ in range(ITEMS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    print(peak)


main()
