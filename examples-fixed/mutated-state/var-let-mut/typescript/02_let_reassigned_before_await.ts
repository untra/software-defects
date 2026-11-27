let activeTenant = "acme";

async function fetchInvoices(tenant: string): Promise<string> {
  await Promise.resolve();
  return `invoices for ${tenant}`;
}

async function main() {
  const acmeRequest = fetchInvoices(activeTenant);
  activeTenant = "globex";
  const globexRequest = fetchInvoices(activeTenant);
  console.log(await acmeRequest);
  console.log(await globexRequest);
}

main();
