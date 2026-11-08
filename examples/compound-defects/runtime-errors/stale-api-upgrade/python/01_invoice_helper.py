INVOICE_CENTS = 450

class Billing:
    def total_cents(self):
        return INVOICE_CENTS

class ReceiptHelper:
    def receipt(self):
        return self.amount()

class Invoice(ReceiptHelper, Billing):
    pass


def main():
    print("receipt", Invoice().receipt())


main()
