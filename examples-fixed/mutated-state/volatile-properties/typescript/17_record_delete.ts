const source = { value: 1 };
const work = { ...source };
delete work.value;
if (!(!("value" in work))) throw new Error("wrong result");
if (!(source.value === 1)) throw new Error("source changed");
console.log("ok");
