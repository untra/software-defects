const source = new Set([1, 2]);
const work = new Set(source);
work.clear();
if (!(work.size === 0))
    throw new Error("wrong result");
if (!(source.size === 2 && source.has(1)))
    throw new Error("source changed");
console.log("ok");
