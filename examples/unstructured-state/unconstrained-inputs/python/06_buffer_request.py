MAX_BUFFER_BYTES = 8


def allocate(requested):
    return bytearray(requested)


try:
    print("bytes", len(allocate(MAX_BUFFER_BYTES + 1)))
except ValueError as error:
    print(error)
