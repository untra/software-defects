const INITIAL_STATUS = "packing";
const NEXT_STATUS = "shipped";
let primary = INITIAL_STATUS;
let replica = INITIAL_STATUS;

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => { open = resolve; });
  return { promise, open };
}

async function copy(ready: Promise<void>): Promise<void> {
  await ready;
  replica = primary;
}

async function main(): Promise<void> {
  const delivery = gate();
  primary = NEXT_STATUS;
  const pending = copy(delivery.promise);
  console.log("status", replica);
  delivery.open();
  await pending;
}

void main();
