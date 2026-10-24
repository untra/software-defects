const MILLIS = 1700000000000;
const PREFIX = "receipt-";
const clock = { millis: MILLIS };
const receipts = new Map<string, string>();

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => { open = resolve; });
  return { promise, open };
}

async function issue(customer: string, ready: Promise<void>): Promise<string> {
  const id = PREFIX + clock.millis;
  const available = !receipts.has(id);
  await ready;
  if (available) receipts.set(id, customer);
  return id;
}

async function main(): Promise<void> {
  const ready = gate();
  const pending = [issue("orchard", ready.promise), issue("harbor", ready.promise)];
  ready.open();
  const ids = await Promise.all(pending);
  console.log("ids", ids.join(" "));
  console.log("receipts", receipts.size);
}

void main();
