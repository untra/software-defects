import json
from typing import TypedDict, cast

class Record(TypedDict):
    title: str

def titles(payload: object) -> list[str]:
    records = cast(list[Record], payload)
    return [record["title"].upper() for record in records]

print(titles(json.loads('{"title":"invoice"}')))
