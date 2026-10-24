trace_id = ""


def handle(name):
    global trace_id
    trace_id = name
    return lambda: f"trace {trace_id}"


def main():
    first = handle("ada")
    second = handle("ben")
    print(first())
    print(second())


main()
