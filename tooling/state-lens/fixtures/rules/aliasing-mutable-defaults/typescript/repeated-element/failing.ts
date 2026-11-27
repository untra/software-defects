const SEAT_COUNT = 3;
const OPEN_SEAT: Readonly<{ holder: string }> = { holder: "open" };
const seats = new Array(SEAT_COUNT).fill(OPEN_SEAT) as { holder: string }[];
seats[0].holder = "ana";
console.log(seats[1].holder);
