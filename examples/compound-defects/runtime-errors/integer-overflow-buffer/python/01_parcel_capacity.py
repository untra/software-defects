import ctypes

LAST_COUNT = 255
PARCEL_BYTE = 65
MARKER = 90


def main():
    count = ctypes.c_uint8(LAST_COUNT)
    count.value += 1

    class Batch(ctypes.Structure):
        _fields_ = [("parcels", ctypes.c_ubyte * count.value), ("label", ctypes.c_ubyte)]

    batch = Batch()
    batch.label = MARKER
    destination = ctypes.cast(ctypes.addressof(batch) + Batch.parcels.offset,
                              ctypes.POINTER(ctypes.c_ubyte))
    destination[0] = PARCEL_BYTE
    print("capacity", count.value, "label", batch.label)


main()
