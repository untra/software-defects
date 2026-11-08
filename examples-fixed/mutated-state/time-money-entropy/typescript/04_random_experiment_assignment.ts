const TREATMENT_SHARE = 0.5;
const PAGE_VIEWS = 20;

const assignments = new Map<string, "A" | "B">();

function variantFor(userId: string): "A" | "B" {
  const existing = assignments.get(userId);
  if (existing) return existing;
  const variant = Math.random() < TREATMENT_SHARE ? "A" : "B";
  assignments.set(userId, variant);
  return variant;
}

const seen = new Set<string>();
for (let view = 0; view < PAGE_VIEWS; view++) seen.add(variantFor("user-42"));
console.log("variants shown to user-42:", seen.size);
