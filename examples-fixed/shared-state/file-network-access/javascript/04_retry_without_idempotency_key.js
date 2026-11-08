const MAX_ATTEMPTS = 3;
const ledger = [];
let dropNextResponse = true;
async function postCharge(body) {
    if (ledger.some(entry => entry.key === body.key))
        return "201 Created";
    ledger.push(body);
    if (dropNextResponse) {
        dropNextResponse = false;
        throw new Error("ECONNRESET");
    }
    return "201 Created";
}
async function chargeWithRetry(amount, operationID) {
    for (let attempt = 1;; attempt++) {
        try {
            return await postCharge({ amount, key: operationID });
        }
        catch (e) {
            if (attempt >= MAX_ATTEMPTS)
                throw e;
        }
    }
}
async function main() {
    console.log(await chargeWithRetry(50, "order-1"));
    console.log("charges recorded:", ledger.length);
}
main();
