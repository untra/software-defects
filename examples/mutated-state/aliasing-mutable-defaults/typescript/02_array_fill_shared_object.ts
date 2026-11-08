const SEAT_COUNT = 3;

const seats = new Array(SEAT_COUNT).fill({ holder: "open" });
seats[0].holder = "ana";
console.log(seats.map((s) => s.holder));
