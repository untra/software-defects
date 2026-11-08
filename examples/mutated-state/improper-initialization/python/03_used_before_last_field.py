class Profile:
    role = None

    def __init__(self, name):
        self.name = name
        self.publish()
        self.role = "staff"

    def publish(self):
        print("role", self.role)


def main():
    Profile("ada")


main()
