class User:
    def __init__(self):
        self.name = ""
        self.admin = False


def load(body):
    user = User()
    for key, value in body.items():
        setattr(user, key, value)
    return user


def main():
    user = load({"name": "ada", "admin": True})
    print(f"admin={user.admin}")


main()
