INITIAL_QUANTITY = 2
PAYLOAD = {}
cart = {"qty": INITIAL_QUANTITY}


def apply(payload):
    cart["qty"] = payload.get("qty")


def main():
    apply(PAYLOAD)
    print("quantity", f"{cart['qty']:.0f}")


main()
