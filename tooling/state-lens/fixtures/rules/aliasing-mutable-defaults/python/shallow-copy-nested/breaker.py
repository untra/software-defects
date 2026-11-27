original = {"tags": ["staff"]}
copied = dict(original)
copied["tags"] = ["staff"]
copied["tags"].append("admin")
print(original["tags"])
