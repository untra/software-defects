import ctypes
import mmap
from tempfile import TemporaryFile

HEADER = b"H"
RECORDS = b"ABC"
REPLACEMENT = ord("Z")
MISSING = -1
SEARCH = ord("Q")


def locate(records, key):
    for index, value in enumerate(records):
        if value == key:
            return index
    return MISSING


def main():
    with TemporaryFile() as handle:
        handle.write(HEADER + RECORDS)
        handle.flush()
        with mmap.mmap(handle.fileno(), 0) as mapping:
            base = ctypes.addressof(ctypes.c_ubyte.from_buffer(mapping))
            records = ctypes.cast(base + len(HEADER), ctypes.POINTER(ctypes.c_ubyte))
            offset = locate(RECORDS, SEARCH)
            records[offset] = REPLACEMENT
            mapping.flush()
        handle.seek(0)
        print("catalog", handle.read().decode())


main()
