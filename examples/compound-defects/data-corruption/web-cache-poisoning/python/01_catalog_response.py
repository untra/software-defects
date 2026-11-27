DEFAULT_HEADING = "Public catalog"
CATALOG_PATH = "/catalog"
pages = {}


def render(path, query):
    if path in pages:
        return pages[path]
    body = query.get("heading", DEFAULT_HEADING)
    pages[path] = body
    return body


def main():
    render(CATALOG_PATH, {"heading": "Members only"})
    print("public page", render(CATALOG_PATH, {}))


main()
