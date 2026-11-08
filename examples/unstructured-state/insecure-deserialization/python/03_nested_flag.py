class User:
    def __init__(self):
        self.name = ""
        self.admin = False


def load(body):
    user = User()
    user.name = body["name"]
    flags = body.get("flags", {})
    user.admin = flags.get("admin", user.admin)
    return user


def main():
    user = load({"name": "ada", "flags": {"admin": True}})
    print(f"admin={user.admin}")


main()
