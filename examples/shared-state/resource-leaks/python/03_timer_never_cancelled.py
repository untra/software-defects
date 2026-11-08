import threading


def main():
    ran = []
    timer = threading.Timer(0.01, lambda: ran.append(1))
    timer.start()
    timer.join()
    print("ran", len(ran))


main()
