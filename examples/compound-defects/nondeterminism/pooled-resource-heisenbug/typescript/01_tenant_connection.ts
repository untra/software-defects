type Connection = { tenant: string };
const FIRST_TENANT = "orchard";
const pool: Connection[] = [{ tenant: "" }];

function borrow(): Connection {
  return pool.pop()!;
}

function release(connection: Connection): void {
  pool.push(connection);
}

function selectTenant(connection: Connection, tenant: string): void {
  connection.tenant = tenant;
}

function main(): void {
  const first = borrow();
  selectTenant(first, FIRST_TENANT);
  release(first);
  const second = borrow();
  console.log("tenant", second.tenant);
  release(second);
}

main();
