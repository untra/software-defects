const prodConfig = { name: "prod", db: { host: "db.internal" } };
const testConfig = { ...prodConfig, name: "test" };
function useLocalDatabase() {
  testConfig.db.host = "localhost";
}
useLocalDatabase();
console.log(prodConfig.db.host);
