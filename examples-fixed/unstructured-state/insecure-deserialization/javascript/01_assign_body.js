const record = { name: "ada", role: "member" };
const body = { name: "ada", role: "admin" };
record.name = body.name;
console.log(`name=${record.name} role=${record.role}`);
