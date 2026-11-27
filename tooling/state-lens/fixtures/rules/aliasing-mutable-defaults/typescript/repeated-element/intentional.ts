const VIEW_COUNT = 3;
const board = { holder: "open" };
const views = new Array(VIEW_COUNT).fill(board);
views[0].holder = "ana";
console.log(views[2].holder);
