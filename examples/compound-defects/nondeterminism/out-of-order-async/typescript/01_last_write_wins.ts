let shown = "";

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => {
    open = resolve;
  });
  return { promise, open };
}

async function load(id: string, ready: Promise<void>): Promise<void> {
  await ready;
  shown = id;
}

async function main(): Promise<void> {
  const earlier = gate();
  const later = gate();
  const first = load("alpha", earlier.promise);
  const second = load("beta", later.promise);
  later.open();
  await second;
  earlier.open();
  await first;
  console.log("shown", shown);
}

void main();
