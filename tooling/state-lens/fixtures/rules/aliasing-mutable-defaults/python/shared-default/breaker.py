def add_tag(item, tags=[]):
    tags = tags + [item]
    return tags


print(add_tag("ada"))
print(add_tag("ben"))
