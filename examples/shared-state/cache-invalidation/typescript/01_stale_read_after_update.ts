const db = new Map<string, string>([["ana", "free"]]);
const cache = new Map<string, string>();

function getPlan(user: string): string {
  const hit = cache.get(user);
  if (hit !== undefined) return hit;
  const plan = db.get(user) ?? "free";
  cache.set(user, plan);
  return plan;
}

function upgrade(user: string, plan: string): void {
  db.set(user, plan);
}

getPlan("ana");
upgrade("ana", "pro");
console.log("ana's plan after paying:", getPlan("ana"));
