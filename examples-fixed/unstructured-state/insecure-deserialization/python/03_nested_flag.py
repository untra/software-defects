class User:
    def __init__(self):
        self.name = ""
        self.admin = False


def load(body):
    user = User()
    user.name = body["name"]
    return user


def main():
    user = load({"name": "ada", "flags": {"admin": True}})
    print(f"admin={user.admin}")


main()
