from typing import Mapping

original: Mapping[str, list] = {"name": ["ada"], "tags": ["staff"]}
copied = dict(original)
copied["tags"].append("admin")
print(original["tags"])
