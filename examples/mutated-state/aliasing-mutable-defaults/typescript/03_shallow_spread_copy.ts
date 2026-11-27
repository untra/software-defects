const prodConfig = { name: "prod", db: { host: "db.internal", pool: 10 } };
const testConfig = { ...prodConfig, name: "test" };
testConfig.db.host = "localhost";
console.log(prodConfig.name, "->", prodConfig.db.host);
