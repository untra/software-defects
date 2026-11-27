const seats = new Array(3).fill({ holder: "open" });
seats[0] = { holder: "open" };
seats[0].holder = "ana";
console.log(seats[1].holder);
