import queue


def main():
    jobs = queue.Queue()
    for n in range(8):
        jobs.put(n)
    print(jobs.maxsize, jobs.qsize())


main()
