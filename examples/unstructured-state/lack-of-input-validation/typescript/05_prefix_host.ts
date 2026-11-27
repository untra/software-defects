const ALLOWED_PREFIX = "https://safe.example";

function allowed(raw: string): boolean {
  return raw.startsWith(ALLOWED_PREFIX);
}

console.log("allowed", allowed("https://safe.example.other.test/a"));
