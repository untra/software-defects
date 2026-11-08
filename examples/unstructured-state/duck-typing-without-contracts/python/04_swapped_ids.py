def link(left, right):
    return f"{left}->{right}"


def main():
    user_id = "u-1"
    order_id = "o-9"
    print(link(order_id, user_id))


main()
