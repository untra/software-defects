const prodConfig = { name: "prod", db: { host: "db.internal" } };
function configure(local: boolean) {
  const testConfig = { ...prodConfig, name: "test" };
  if (local) {
    testConfig.db.host = "localhost";
    return testConfig;
  }
  console.log(prodConfig.db.host);
  return testConfig;
}
configure(true);
