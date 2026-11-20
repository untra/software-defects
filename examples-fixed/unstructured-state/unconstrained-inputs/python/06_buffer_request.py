MAX_BUFFER_BYTES = 8


def allocate(requested):
    if type(requested) is not int or not 0 <= requested <= MAX_BUFFER_BYTES:
        raise ValueError("buffer rejected")
    return bytearray(requested)


try:
    print("bytes", len(allocate(MAX_BUFFER_BYTES + 1)))
except ValueError as error:
    print(error)
