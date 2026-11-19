import threading

WITHDRAWALS = 5


def main():
    balance = 100
    start = threading.Barrier(WITHDRAWALS)
    lock = threading.Lock()

    def withdraw():
        nonlocal balance
        start.wait()
        with lock:
            if balance >= 100:
                balance -= 100

    threads = [threading.Thread(target=withdraw) for _ in range(WITHDRAWALS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    print(balance)


main()
