const source = [3, 1, 1, 2];
const work = [...source];
work.reverse();
if (JSON.stringify(work) !== JSON.stringify([2, 1, 1, 3]))
    throw new Error("wrong result");
if (JSON.stringify(source) !== "[3,1,1,2]")
    throw new Error("source changed");
console.log("ok");
