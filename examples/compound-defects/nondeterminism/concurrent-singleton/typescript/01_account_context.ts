class AccountContext {
  user = "";

  async show(user: string, ready: Promise<void>): Promise<string> {
    this.user = user;
    await ready;
    return this.user;
  }
}

const context = new AccountContext();

function gate(): { promise: Promise<void>; open: () => void } {
  let open: () => void = () => undefined;
  const promise = new Promise<void>((resolve) => { open = resolve; });
  return { promise, open };
}

async function main(): Promise<void> {
  const firstReady = gate();
  const first = context.show("orchard", firstReady.promise);
  await context.show("harbor", Promise.resolve());
  firstReady.open();
  console.log("orchard received", await first);
}

void main();
