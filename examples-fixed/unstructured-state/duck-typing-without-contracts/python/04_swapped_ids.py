def link(*, user_id, order_id):
    return f"{user_id}->{order_id}"


def main():
    user_id = "u-1"
    order_id = "o-9"
    print(link(user_id=user_id, order_id=order_id))


main()
