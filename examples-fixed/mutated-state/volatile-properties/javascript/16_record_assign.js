const source = { value: 1 };
const work = { ...source };
work.value = 9;
if (!(work.value === 9))
    throw new Error("wrong result");
if (!(source.value === 1))
    throw new Error("source changed");
console.log("ok");
