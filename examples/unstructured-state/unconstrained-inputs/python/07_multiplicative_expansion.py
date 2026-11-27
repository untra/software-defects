MAX_COPIES = 4
MAX_INPUT_ITEMS = 4
MAX_OUTPUT_ITEMS = 6


def expand(items, copies):
    if type(copies) is not int or not 0 <= copies <= MAX_COPIES or len(items) > MAX_INPUT_ITEMS:
        raise ValueError("input rejected")
    output = []
    for _ in range(copies):
        output.extend(items)
    return output


try:
    print("expanded", len(expand(["a", "b", "c"], 3)))
except ValueError as error:
    print(error)
