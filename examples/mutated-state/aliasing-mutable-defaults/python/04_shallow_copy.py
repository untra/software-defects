def main():
    original = {"name": "ada", "tags": ["staff"]}
    copy = original.copy()
    copy["tags"].append("admin")
    print(original["tags"])


main()
