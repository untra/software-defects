signed = {"Authorization": "Bearer secret"}


def normalize(headers):
    return headers


public = dict(normalize(signed))
public["Authorization"] = ""
print(signed)
