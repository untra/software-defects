class User:
    def __init__(self, name, admin=False):
        self.name = name
        self.admin = admin


def main():
    data = {"name": "ada", "admin": True}
    user = User(name=data["name"])
    print(f"admin={user.admin}")


main()
