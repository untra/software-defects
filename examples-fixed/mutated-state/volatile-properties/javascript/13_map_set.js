const source = new Map([[1, 10], [2, 20]]);
const work = new Map(source);
const result = work.set(1, 99);
if (!(work.get(1) === 99 && result === work))
    throw new Error("wrong result");
if (!(source.size === 2 && source.has(1) && source.get(1) === 10))
    throw new Error("source changed");
console.log("ok");
