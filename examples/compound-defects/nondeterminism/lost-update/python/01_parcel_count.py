import threading

CLERKS = 2
WAIT_SECONDS = 2
parcels = 0


def main():
    loaded = threading.Barrier(CLERKS, timeout=WAIT_SECONDS)

    def receive():
        global parcels
        current = parcels
        loaded.wait()
        current += 1
        parcels = current

    clerks = [threading.Thread(target=receive) for _ in range(CLERKS)]
    for clerk in clerks:
        clerk.start()
    for clerk in clerks:
        clerk.join(WAIT_SECONDS)
    print("parcels", parcels)


main()
