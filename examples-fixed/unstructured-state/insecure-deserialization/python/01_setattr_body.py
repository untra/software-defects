class User:
    def __init__(self):
        self.name = ""
        self.admin = False


def load(body):
    user = User()
    if isinstance(body.get("name"), str):
        user.name = body["name"]
    return user


def main():
    user = load({"name": "ada", "admin": True})
    print(f"admin={user.admin}")


main()
