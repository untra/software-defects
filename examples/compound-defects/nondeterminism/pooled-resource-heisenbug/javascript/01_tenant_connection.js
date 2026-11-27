const FIRST_TENANT = "orchard";
const pool = [{ tenant: "" }];
function borrow() {
    return pool.pop();
}
function release(connection) {
    pool.push(connection);
}
function selectTenant(connection, tenant) {
    connection.tenant = tenant;
}
function main() {
    const first = borrow();
    selectTenant(first, FIRST_TENANT);
    release(first);
    const second = borrow();
    console.log("tenant", second.tenant);
    release(second);
}
main();
