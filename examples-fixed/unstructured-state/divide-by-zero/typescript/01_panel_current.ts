const voltage = 12;
const resistance = 0;
const current = resistance === 0 ? null : voltage / resistance;
console.log("current", current);
