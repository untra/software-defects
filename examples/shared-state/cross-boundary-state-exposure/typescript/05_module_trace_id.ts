const DB_LATENCY_MS = 10;

let traceId = "";

function log(message: string): void {
  console.log(`[${traceId}] ${message}`);
}

async function handle(requestId: string, action: string): Promise<void> {
  traceId = requestId;
  await new Promise((resolve) => setTimeout(resolve, DB_LATENCY_MS));
  log(action);
}

async function main() {
  await Promise.all([handle("req-1", "refund issued"), handle("req-2", "profile viewed")]);
}

main();
