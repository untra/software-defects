ALLOWED = "shop.example"


def allowed(host):
    return host.startswith(ALLOWED)


def main():
    print(allowed("shop.example.other.test"))


main()
