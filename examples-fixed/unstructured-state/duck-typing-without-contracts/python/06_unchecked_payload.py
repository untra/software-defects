import json

def titles(payload: object) -> list[str]:
    if not isinstance(payload, list) or not all(
        isinstance(record, dict) and isinstance(record.get("title"), str)
        for record in payload
    ):
        return ["invalid records"]
    return [record["title"].upper() for record in payload]

print(titles(json.loads('{"title":"invoice"}')))
