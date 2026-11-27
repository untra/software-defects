type Config = { name: string; db: { host: string; pool: number } };
const prodConfig: Readonly<Config> = { name: "prod", db: { host: "db.internal", pool: 10 } };
const testConfig = { ...prodConfig, name: "test" };
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
