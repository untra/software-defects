import threading

WITHDRAWALS = 5


def main():
    balance = 100
    start = threading.Barrier(WITHDRAWALS)
    checked = threading.Barrier(WITHDRAWALS)

    def withdraw():
        nonlocal balance
        start.wait()
        allowed = balance >= 100
        checked.wait()
        if allowed:
            balance = balance - 100

    threads = [threading.Thread(target=withdraw) for _ in range(WITHDRAWALS)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    print(balance)


main()
