import threading


def main():
    rows = None
    created = []
    lock = threading.Lock()

    def load():
        nonlocal rows
        with lock:
            if rows is None:
                fresh = []
                created.append(fresh)
                rows = fresh

    threads = [threading.Thread(target=load) for _ in range(2)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    print(len(created))


main()
