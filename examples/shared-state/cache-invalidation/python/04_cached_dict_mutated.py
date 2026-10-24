cache = {}


def profile(name):
    if name not in cache:
        cache[name] = {"name": name, "role": "staff"}
    return cache[name]


def main():
    ada = profile("ada")
    ada["role"] = "admin"
    print(profile("ada")["role"])


main()
