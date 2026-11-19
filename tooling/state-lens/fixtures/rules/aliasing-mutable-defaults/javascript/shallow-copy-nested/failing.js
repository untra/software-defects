const prodConfig = Object.freeze({ name: "prod", db: { host: "db.internal" } });
const testConfig = { ...prodConfig, name: "test" };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
