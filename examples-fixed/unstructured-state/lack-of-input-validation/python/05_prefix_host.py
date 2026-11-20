ALLOWED = "shop.example"


def allowed(host):
    return host == ALLOWED


def main():
    print(allowed("shop.example.other.test"))


main()
