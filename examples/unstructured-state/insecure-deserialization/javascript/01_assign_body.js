const record = { name: "ada", role: "member" };
const body = { name: "ada", role: "admin" };
Object.assign(record, body);
console.log(`name=${record.name} role=${record.role}`);
