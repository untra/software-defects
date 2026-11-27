import queue

CAPACITY = 8


def main():
    jobs = queue.Queue(maxsize=CAPACITY)
    for n in range(CAPACITY):
        jobs.put(n)
    print(jobs.maxsize, jobs.qsize())


main()
