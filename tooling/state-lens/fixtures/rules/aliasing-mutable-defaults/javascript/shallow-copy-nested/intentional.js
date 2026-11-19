const shared = { pool: { size: 10 } };
const tenant = { ...shared, name: "tenant" };
tenant.pool.size = 20;
console.log(shared.pool.size);
