from typing import Mapping

original: Mapping[str, list] = {"name": ["ada"], "tags": ["staff"]}
copied = {**original, "tags": list(original["tags"])}
copied["tags"].append("admin")
print(original["tags"])
