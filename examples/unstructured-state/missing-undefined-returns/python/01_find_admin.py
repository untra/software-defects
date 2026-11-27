def require_admin(people):
    for person in people:
        if person["role"] == "admin":
            return person

try:
    print("admin", require_admin([{"name": "ada", "role": "staff"}]))
except LookupError as error:
    print(error)
