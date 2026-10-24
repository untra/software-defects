import json

BODY = '{"name":"mira","admin":true}'
account = {"name": "mira", "admin": False}


def receive_profile(body):
    profile = json.loads(body)
    account.update(profile)


def main():
    receive_profile(BODY)
    print("admin", account["admin"])


main()
