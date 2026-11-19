declare function reconfigure(config: object): void;
const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig, name: "test" };
reconfigure(testConfig);
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
