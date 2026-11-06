const RESERVED_KIND = "reserved";
const RUSH_KIND = "rush";
const ASSIGNED_SEAT = "b12";
function issue(kind) {
    const ticket = { kind };
    if (kind === RESERVED_KIND)
        ticket.seat = ASSIGNED_SEAT;
    return ticket;
}
function printSeat(ticket) {
    return ticket.seat.toUpperCase();
}
function main() {
    console.log("seat", printSeat(issue(RUSH_KIND)));
}
main();
