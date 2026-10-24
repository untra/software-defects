def find_admin(people):
    for person in people:
        if person["role"] == "admin":
            return person


def main():
    found = find_admin([{"name": "ada", "role": "staff"}])
    print(found["name"])


main()
