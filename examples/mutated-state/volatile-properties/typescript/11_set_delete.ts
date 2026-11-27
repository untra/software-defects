const source = new Set([1, 2]);
const work = source;
const result = work.delete(1);
if (!(!work.has(1) && result === true)) throw new Error("wrong result");
if (!(source.size === 2 && source.has(1))) throw new Error("source changed");
console.log("ok");
