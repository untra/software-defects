const source = [3, 1, 1, 2];
const work = source;
work.fill(9, 1, 3);
if (JSON.stringify(work) !== JSON.stringify([3, 9, 9, 2]))
    throw new Error("wrong result");
if (JSON.stringify(source) !== "[3,1,1,2]")
    throw new Error("source changed");
console.log("ok");
