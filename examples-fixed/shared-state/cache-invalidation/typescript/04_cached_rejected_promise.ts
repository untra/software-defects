let configServerUp = false;
const pending = new Map<string, Promise<string>>();

async function fetchConfig(name: string): Promise<string> {
  if (!configServerUp) throw new Error("config server unavailable");
  return `${name}=enabled`;
}

function getConfig(name: string): Promise<string> {
  if (!pending.has(name)) pending.set(name, fetchConfig(name).catch(error => { pending.delete(name); throw error; }));
  return pending.get(name)!;
}

async function main() {
  await getConfig("checkout").catch(() => {});
  configServerUp = true;
  try {
    console.log(await getConfig("checkout"));
  } catch (e) {
    console.log("after recovery:", (e as Error).message);
  }
}

main();
