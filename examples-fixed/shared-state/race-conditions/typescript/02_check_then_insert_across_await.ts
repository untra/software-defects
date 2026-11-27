const HASH_LATENCY_MS = 5;

const accounts: { email: string; hash: string }[] = [];

async function hashPassword(password: string): Promise<string> {
  await new Promise((resolve) => setTimeout(resolve, HASH_LATENCY_MS));
  return `hashed:${password.length}`;
}

async function signup(email: string, password: string): Promise<boolean> {
  const hash = await hashPassword(password);
  if (accounts.some((a) => a.email === email)) return false;
  accounts.push({ email, hash });
  return true;
}

async function main() {
  await Promise.all([signup("ana@example.com", "pw1"), signup("ana@example.com", "pw2")]);
  console.log("accounts for ana@example.com:", accounts.filter((a) => a.email === "ana@example.com").length);
}

main();
