const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig, name: "test" };
testConfig.db = { host: "db.internal" };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
