const tray = {
  fill(value: { holder: string }) {
    return [value, { ...value }];
  },
};
const seats = tray.fill({ holder: "open" });
seats[0].holder = "ana";
console.log(seats[1].holder);
