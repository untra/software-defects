def with_tax(lines):
    lines.append({"sku": "tax", "cents": 5})
    return lines


def main():
    cart = [{"sku": "pen", "cents": 40}]
    print(len(with_tax(cart)), len(cart))


main()
