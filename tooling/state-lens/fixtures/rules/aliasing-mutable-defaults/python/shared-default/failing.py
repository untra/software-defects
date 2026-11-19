from typing import Sequence


def add_tag(item, tags: Sequence[str] = []):
    tags.append(item)
    return tags


print(add_tag("ada"))
print(add_tag("ben"))
