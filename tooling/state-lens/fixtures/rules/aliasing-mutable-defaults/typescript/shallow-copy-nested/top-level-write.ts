const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig };
testConfig.name = "test";
console.log(prodConfig.name, prodConfig.db.host);
