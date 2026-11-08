import threading

WAIT_SECONDS = 2
BEFORE = [1, 2, 3]
AFTER = [10, 20, 30]
slots = BEFORE.copy()


def main():
    scanned = threading.Event()
    updated = threading.Event()
    totals = []

    def count():
        total = 0
        for index in range(len(slots)):
            total += slots[index]
            if index == 0:
                scanned.set()
                updated.wait(WAIT_SECONDS)
        totals.append(total)

    def replenish():
        scanned.wait(WAIT_SECONDS)
        slots[:] = AFTER
        updated.set()

    workers = [threading.Thread(target=count), threading.Thread(target=replenish)]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join(WAIT_SECONDS)
    print("units", totals[0])


main()
