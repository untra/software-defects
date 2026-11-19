const seats = new Array(3).fill({ holder: "open" });
const index = Number(process.argv[2]);
seats[index].holder = "ana";
console.log(seats[1].holder);
