const ALLOWED_PREFIX = "https://safe.example";

function allowed(raw: string): boolean {
  try { return new URL(raw).origin === new URL(ALLOWED_PREFIX).origin; }
  catch { return false; }
}

console.log("allowed", allowed("https://safe.example.other.test/a"));
