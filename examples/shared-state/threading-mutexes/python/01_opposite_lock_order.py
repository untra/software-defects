import threading


def main():
    lock_a = threading.Lock()
    lock_b = threading.Lock()
    held = {"a": threading.Event(), "b": threading.Event()}
    results = {}
    finished = threading.Barrier(2)

    def run(name, first, second, own, other):
        first.acquire()
        held[own].set()
        held[other].wait()
        got = second.acquire(timeout=0.05)
        results[name] = "blocked" if not got else "entered"
        if got:
            second.release()
        finished.wait()
        first.release()

    left = threading.Thread(target=run, args=("a", lock_a, lock_b, "a", "b"))
    right = threading.Thread(target=run, args=("b", lock_b, lock_a, "b", "a"))
    left.start()
    right.start()
    left.join()
    right.join()
    print("a", results["a"])
    print("b", results["b"])


main()
