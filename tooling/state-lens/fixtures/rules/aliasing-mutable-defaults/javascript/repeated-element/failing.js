const SEAT_COUNT = 3;
const defaults = Object.freeze({ seat: { holder: "open" } });
const seats = new Array(SEAT_COUNT).fill(defaults.seat);
seats[0].holder = "ana";
console.log(seats[1].holder);
