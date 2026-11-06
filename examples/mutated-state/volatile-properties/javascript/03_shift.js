const source = [3, 1, 1, 2];
const work = source;
const result = work.shift();
if (JSON.stringify(work) !== JSON.stringify([1, 1, 2]))
    throw new Error("wrong result");
if (JSON.stringify(result) !== JSON.stringify(3))
    throw new Error("wrong return");
if (JSON.stringify(source) !== "[3,1,1,2]")
    throw new Error("source changed");
console.log("ok");
