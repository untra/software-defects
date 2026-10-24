LINE_CENTS = 250
FORM_CENTS = "275"

class Line:
    def __init__(self):
        self.cents = LINE_CENTS

    def edit(self, cents):
        self.cents = cents


def format_lines(lines):
    return [f"{line.cents:.2f}" for line in lines]


def main():
    line = Line()
    lines = [line]
    line.edit(FORM_CENTS)
    print("receipt", format_lines(lines))


main()
