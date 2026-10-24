const QUERY_LATENCY_MS = 10;

let currentUser: string | null = null;

async function loadAccount(): Promise<string> {
  await new Promise((resolve) => setTimeout(resolve, QUERY_LATENCY_MS));
  return `account of ${currentUser}`;
}

async function handleRequest(user: string): Promise<string> {
  currentUser = user;
  return `${user} received ${await loadAccount()}`;
}

async function main() {
  const responses = await Promise.all([handleRequest("alice"), handleRequest("bob")]);
  responses.forEach((r) => console.log(r));
}

main();
