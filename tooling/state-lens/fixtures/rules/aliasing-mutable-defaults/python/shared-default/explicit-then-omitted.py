def add_tag(item, tags=[]):
    tags.append(item)
    return tags


add_tag("ada", ["staff"])
add_tag("ben")
print(add_tag("cy"))
