def main():
    record = {"name": "ada", "admin": False}
    incoming = {"admin": True}
    if isinstance(incoming.get("name"), str):
        record["name"] = incoming["name"]
    print(f"admin={record['admin']}")


main()
