RESERVED_KIND = "reserved"
RUSH_KIND = "rush"
ASSIGNED_SEAT = "b12"

class Ticket:
    def __init__(self, kind):
        if kind == RESERVED_KIND:
            self.seat = ASSIGNED_SEAT


def print_seat(ticket):
    return ticket.seat.upper()


def main():
    print("seat", print_seat(Ticket(RUSH_KIND)))


main()
