import ctypes

NAME_BYTES = 4
STATUS_BYTES = 4
NAME = b"ORCHARD"
STATUS = b"OPEN"

class Customer(ctypes.Structure):
    _fields_ = [("name", ctypes.c_char * NAME_BYTES),
                ("status", ctypes.c_char * STATUS_BYTES)]

customer = Customer()


def store_name(name):
    destination = ctypes.addressof(customer) + Customer.name.offset
    ctypes.memmove(destination, name, len(name))


def main():
    customer.status = STATUS
    store_name(NAME)
    print("status", customer.status.decode())


main()
