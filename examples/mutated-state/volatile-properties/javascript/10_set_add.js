const source = new Set([1, 2]);
const work = source;
const result = work.add(3);
if (!(work.has(3) && result === work))
    throw new Error("wrong result");
if (!(source.size === 2 && source.has(1)))
    throw new Error("source changed");
console.log("ok");
