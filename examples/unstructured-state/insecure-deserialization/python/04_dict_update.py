def main():
    record = {"name": "ada", "admin": False}
    record.update({"admin": True})
    print(f"admin={record['admin']}")


main()
