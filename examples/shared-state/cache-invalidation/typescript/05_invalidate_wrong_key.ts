const db = new Map<string, string>([["ana@example.com", "Ana"]]);
const cache = new Map<string, string>();

const normalize = (email: string) => email.trim().toLowerCase();

function displayName(email: string): string {
  const key = normalize(email);
  const hit = cache.get(key);
  if (hit !== undefined) return hit;
  const name = db.get(key) ?? "unknown";
  cache.set(key, name);
  return name;
}

function rename(email: string, name: string): void {
  db.set(normalize(email), name);
  cache.delete(email);
}

displayName("Ana@Example.com");
rename("Ana@Example.com", "Ana Lopez");
console.log("greeting:", displayName("ana@example.com"));
