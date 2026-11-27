const OPENING_BALANCE = 100;
const POSTING_CAPACITY = 1;
let balance = OPENING_BALANCE;
let inFlight = 0;

async function postRefund(refund: number): Promise<void> {
  inFlight++;
  try {
    if (inFlight > POSTING_CAPACITY) throw new Error("ledger busy");
    await Promise.resolve();
    const credit = refund >>> 0;
    balance += credit;
  } finally {
    inFlight--;
  }
}

async function main(): Promise<void> {
  const REFUND = -1;
  const postings = await Promise.allSettled([postRefund(REFUND), postRefund(REFUND)]);
  console.log("busy", postings.filter((posting) => posting.status === "rejected").length);
  console.log("balance", balance);
}

void main();
