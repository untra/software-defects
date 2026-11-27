import json

BODY = '{"colour":"blue"}'

class RecordLoader:
    def load(self, body):
        for key, value in json.loads(body).items():
            setattr(self, key, value)

class Paint(RecordLoader):
    pass


def main():
    paint = Paint()
    paint.load(BODY)
    print("paint", paint.color.upper())


main()
