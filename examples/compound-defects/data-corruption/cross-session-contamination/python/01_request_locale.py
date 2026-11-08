DEFAULT_LOCALE = "en-US"
context = {"locale": DEFAULT_LOCALE}


def render(request):
    if "locale" in request:
        context["locale"] = request["locale"]
    return context["locale"]


def main():
    render({"locale": "staff-preview"})
    print("next locale", render({}))


main()
