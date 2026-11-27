let activeTenant = "acme";

async function fetchInvoices(): Promise<string> {
  await Promise.resolve();
  return `invoices for ${activeTenant}`;
}

async function main() {
  const acmeRequest = fetchInvoices();
  activeTenant = "globex";
  const globexRequest = fetchInvoices();
  console.log(await acmeRequest);
  console.log(await globexRequest);
}

main();
