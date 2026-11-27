const source = new Map([[1, 10], [2, 20]]);
const work = source;
work.clear();
if (!(work.size === 0))
    throw new Error("wrong result");
if (!(source.size === 2 && source.has(1) && source.get(1) === 10))
    throw new Error("source changed");
console.log("ok");
