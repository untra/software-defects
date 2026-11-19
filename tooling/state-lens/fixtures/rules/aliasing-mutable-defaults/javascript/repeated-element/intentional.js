const board = { holder: "open" };
const views = new Array(3).fill(board);
views[0].holder = "ana";
console.log(views[2].holder);
