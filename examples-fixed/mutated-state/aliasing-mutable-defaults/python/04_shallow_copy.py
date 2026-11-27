def main():
    original = {"name": "ada", "tags": ["staff"]}
    copy = {**original, "tags": list(original["tags"])}
    copy["tags"].append("admin")
    print(original["tags"])


main()
