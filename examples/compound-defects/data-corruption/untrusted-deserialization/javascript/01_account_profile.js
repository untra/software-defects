const account = { name: "mira", admin: false };
function receiveProfile(body) {
    const profile = JSON.parse(body);
    Object.assign(account, profile);
}
function main() {
    const REQUEST_BODY = '{"name":"mira","admin":true}';
    receiveProfile(REQUEST_BODY);
    console.log("admin", account.admin);
}
main();
