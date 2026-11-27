const source = [3, 1, 1, 2];
const work = [...source];
work.sort((a, b) => a - b);
if (JSON.stringify(work) !== JSON.stringify([1, 1, 2, 3])) throw new Error("wrong result");
if (JSON.stringify(source) !== "[3,1,1,2]") throw new Error("source changed");
console.log("ok");
