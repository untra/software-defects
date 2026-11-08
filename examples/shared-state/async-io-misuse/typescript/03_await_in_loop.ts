const SERVICE_LATENCY_MS = 30;
const SERVICES = ["billing", "shipping", "inventory", "reviews", "recommendations"];

async function fetchWidget(service: string): Promise<string> {
  await new Promise((resolve) => setTimeout(resolve, SERVICE_LATENCY_MS));
  return `${service} widget`;
}

async function renderHomepage(): Promise<string[]> {
  const widgets: string[] = [];
  for (const service of SERVICES) {
    widgets.push(await fetchWidget(service));
  }
  return widgets;
}

async function main() {
  const start = Date.now();
  await renderHomepage();
  const elapsed = Date.now() - start;
  console.log("homepage took longer than", SERVICES.length - 1, "round trips:", elapsed >= SERVICE_LATENCY_MS * (SERVICES.length - 1));
}

main();
