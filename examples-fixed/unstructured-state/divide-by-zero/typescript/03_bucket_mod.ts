const jobs = 10;
const buckets = 0;
const remainder = buckets === 0 ? null : jobs % buckets;
console.log("remainder", remainder);
