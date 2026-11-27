import { reset } from "./reset";
const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig, name: "test" };
reset(testConfig);
testConfig.db.host = "localhost";
console.log(prodConfig.db.host);
