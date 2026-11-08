const TREATMENT_SHARE = 0.5;
const PAGE_VIEWS = 20;

function variantFor(userId: string): "A" | "B" {
  return Math.random() < TREATMENT_SHARE ? "A" : "B";
}

const seen = new Set<string>();
for (let view = 0; view < PAGE_VIEWS; view++) seen.add(variantFor("user-42"));
console.log("variants shown to user-42:", seen.size);
