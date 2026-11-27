class Celsius:
    def __init__(self, value):
        self.value = value


class Kelvin:
    def __init__(self, value):
        self.value = value


def to_fahrenheit(reading):
    return reading.value * 9 / 5 + 32


def main():
    print(to_fahrenheit(Kelvin(300)))


main()
