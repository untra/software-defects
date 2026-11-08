const CAPACITY = 3;
let seatsTaken = 0;

function reserve(name: string): string {
  if (seatsTaken++ <= CAPACITY) return `${name}: confirmed`;
  return `${name}: waitlisted`;
}

for (const guest of ["ana", "ben", "cy", "dee", "eli"]) console.log(reserve(guest));
console.log("seats taken:", seatsTaken);
