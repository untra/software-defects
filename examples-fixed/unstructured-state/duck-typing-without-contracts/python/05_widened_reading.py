FAHRENHEIT_SCALE = 1.8
FAHRENHEIT_OFFSET = 32
KELVIN_OFFSET = 273.15

class Celsius:
    def __init__(self, value):
        self.value = value


class Kelvin:
    def __init__(self, value):
        self.value = value


def to_fahrenheit(reading):
    if isinstance(reading, Kelvin):
        celsius = reading.value - KELVIN_OFFSET
    elif isinstance(reading, Celsius):
        celsius = reading.value
    else:
        raise TypeError("unsupported temperature scale")
    return celsius * FAHRENHEIT_SCALE + FAHRENHEIT_OFFSET


def main():
    print(to_fahrenheit(Kelvin(300)))


main()
