const prodConfig = { name: "prod", db: { host: "db.internal" } };
let testConfig = { ...prodConfig };
testConfig = { name: "test", db: { host: "db.internal" } };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
