import threading


def main():
    lock_a = threading.Lock()
    lock_b = threading.Lock()
    results = {}

    def run(name):
        with lock_a:
            with lock_b:
                results[name] = "entered"

    workers = [threading.Thread(target=run, args=(name,)) for name in ("a", "b")]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join()
    print("a", results["a"])
    print("b", results["b"])


main()
