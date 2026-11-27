const source = [3, 1, 1, 2];
const work = source;
work.copyWithin(0, 2);
if (JSON.stringify(work) !== JSON.stringify([1, 2, 1, 2]))
    throw new Error("wrong result");
if (JSON.stringify(source) !== "[3,1,1,2]")
    throw new Error("source changed");
console.log("ok");
