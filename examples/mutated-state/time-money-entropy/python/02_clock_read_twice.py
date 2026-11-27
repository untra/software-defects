from datetime import datetime

DATE_LAYOUT = "%Y-%m-%d"
ticks = []


def clock():
    return ticks.pop(0)


def report_key():
    date = clock().strftime(DATE_LAYOUT)
    hour = clock().hour
    return f"{date}-h{hour:02d}"


def main():
    ticks.append(datetime(2026, 10, 23, 23, 59, 59))
    ticks.append(datetime(2026, 10, 24, 0, 0, 0))
    print("report key:", report_key())


main()
