const SEAT_COUNT = 3;

const seats = Array.from({ length: SEAT_COUNT }, () => ({ holder: "open" }));
seats[0].holder = "ana";
console.log(seats.map((s) => s.holder));
