const prodConfig = Object.freeze({ name: "prod", db: { host: "db.internal" } });
const testConfig = { ...prodConfig, name: "test", db: { ...prodConfig.db } };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
