from typing import Mapping

signed: Mapping[str, str] = {"Authorization": "Bearer secret"}


def normalize(headers):
    return headers


public = normalize(signed)
public["Authorization"] = ""
print(signed)
