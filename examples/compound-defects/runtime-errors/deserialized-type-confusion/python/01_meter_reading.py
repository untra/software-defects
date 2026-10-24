import json

BODY = '{"reading":"10"}'
INITIAL_READING = 0

class Meter:
    def __init__(self):
        self.reading = INITIAL_READING

    def load(self, body):
        record = json.loads(body)
        self.reading = record["reading"]

    def display(self):
        return f"{self.reading:.1f}"


def main():
    meter = Meter()
    meter.load(BODY)
    print("reading", meter.display())


main()
