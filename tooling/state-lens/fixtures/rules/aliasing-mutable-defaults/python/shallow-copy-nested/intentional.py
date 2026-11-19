base = {"pool": {"size": 10}}
tenant = base.copy()
tenant["pool"]["size"] = 20
print(base["pool"])
