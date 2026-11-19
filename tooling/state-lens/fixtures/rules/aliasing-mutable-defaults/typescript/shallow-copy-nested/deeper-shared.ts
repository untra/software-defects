const prodConfig = { db: { host: "db.internal", tls: { verify: true } } };
const testConfig = { ...prodConfig, db: { ...prodConfig.db } };
testConfig.db.host = "localhost";
testConfig.db.tls.verify = false;
console.log(prodConfig.db.host, prodConfig.db.tls.verify);
