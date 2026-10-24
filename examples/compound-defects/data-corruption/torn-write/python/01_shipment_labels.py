import ctypes
import threading

LABEL_BYTES = 4
FOOTER_BYTES = 4
FIRST_OFFSET = 2
SECOND_OFFSET = 3
FIRST_LABEL = b"AAAA"
SECOND_LABEL = b"BBBB"
WAIT_SECONDS = 2

class Shipment(ctypes.Structure):
    _fields_ = [("label", ctypes.c_char * LABEL_BYTES),
                ("footer", ctypes.c_char * FOOTER_BYTES)]

shipment = Shipment()


def write_at(offset, data):
    destination = ctypes.addressof(shipment) + Shipment.label.offset + offset
    ctypes.memmove(destination, data, len(data))


def main():
    shipment.label = b"...."
    shipment.footer = b"...."
    started = threading.Event()
    delivered = threading.Event()

    def first():
        write_at(FIRST_OFFSET, FIRST_LABEL[:1])
        started.set()
        delivered.wait(WAIT_SECONDS)
        write_at(FIRST_OFFSET + 1, FIRST_LABEL[1:])

    def second():
        started.wait(WAIT_SECONDS)
        write_at(SECOND_OFFSET, SECOND_LABEL)
        delivered.set()

    workers = [threading.Thread(target=first), threading.Thread(target=second)]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join(WAIT_SECONDS)
    print("label", shipment.label.decode(), "footer", shipment.footer.decode())


main()
