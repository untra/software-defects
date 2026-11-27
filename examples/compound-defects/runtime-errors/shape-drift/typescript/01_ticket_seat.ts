const RESERVED_KIND = "reserved";
const RUSH_KIND = "rush";
const ASSIGNED_SEAT = "b12";

type Ticket = { kind: string; seat?: string };

function issue(kind: string): Ticket {
  const ticket: Ticket = { kind };
  if (kind === RESERVED_KIND) ticket.seat = ASSIGNED_SEAT;
  return ticket;
}

function printSeat(ticket: Ticket): string {
  return ticket.seat.toUpperCase();
}

function main(): void {
  console.log("seat", printSeat(issue(RUSH_KIND)));
}

main();
