const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig, name: "test" };
try {
  testConfig.db.host = "localhost";
  throw new Error("probe");
} catch {
  console.log(prodConfig.db.host);
}
