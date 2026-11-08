const source = [3, 1, 1, 2];
const work = source;
const result = work.push(9);
if (JSON.stringify(work) !== JSON.stringify([3, 1, 1, 2, 9])) throw new Error("wrong result");
if (JSON.stringify(result) !== JSON.stringify(5)) throw new Error("wrong return");
if (JSON.stringify(source) !== "[3,1,1,2]") throw new Error("source changed");
console.log("ok");
