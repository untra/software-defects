const source = new Map([[1, 10], [2, 20]]);
const work = source;
const result = work.delete(1);
if (!(!work.has(1) && result === true))
    throw new Error("wrong result");
if (!(source.size === 2 && source.has(1) && source.get(1) === 10))
    throw new Error("source changed");
console.log("ok");
