import ctypes
import sys

CODE_BYTES = 2
STATUS_BYTES = 2
PAYLOAD = b"ABCD"
STATUS = b"OK"

class Parcel(ctypes.Structure):
    _fields_ = [("code", ctypes.c_ubyte * CODE_BYTES),
                ("status", ctypes.c_ubyte * STATUS_BYTES)]

parcel = Parcel()


def set_code(value):
    address = ctypes.addressof(parcel) + Parcel.code.offset
    code = ctypes.cast(address, ctypes.POINTER(ctypes.c_uint32))
    code[0] = value


def main():
    parcel.status[:] = STATUS
    set_code(int.from_bytes(PAYLOAD, sys.byteorder))
    print("status", bytes(parcel.status).decode())


main()
