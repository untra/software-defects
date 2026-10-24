import threading


def main():
    rows = None
    created = []
    barrier = threading.Barrier(2)

    def load():
        nonlocal rows
        if rows is None:
            barrier.wait()
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
