STARTING_CENTS = 1250
FORM_CENTS = "1425"

class Invoice:
    def __init__(self):
        self.cents = STARTING_CENTS

    def apply(self, form):
        self.cents = form["cents"]

    def display(self):
        return f"{self.cents:.2f}"


def main():
    invoice = Invoice()
    invoice.apply({"cents": FORM_CENTS})
    print("cents", invoice.display())


main()
