const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { db: { ...prodConfig.db }, ...prodConfig };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
