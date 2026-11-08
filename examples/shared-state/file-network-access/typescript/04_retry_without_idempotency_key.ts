const MAX_ATTEMPTS = 3;

const ledger: { amount: number; key?: string }[] = [];
let dropNextResponse = true;

async function postCharge(body: { amount: number }): Promise<string> {
  ledger.push(body);
  if (dropNextResponse) {
    dropNextResponse = false;
    throw new Error("ECONNRESET");
  }
  return "201 Created";
}

async function chargeWithRetry(amount: number): Promise<string> {
  for (let attempt = 1; ; attempt++) {
    try {
      return await postCharge({ amount });
    } catch (e) {
      if (attempt >= MAX_ATTEMPTS) throw e;
    }
  }
}

async function main() {
  console.log(await chargeWithRetry(50));
  console.log("charges recorded:", ledger.length);
}

main();
