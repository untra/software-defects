const scores = [90, 85, 77];
const ranked: string[] = [];

for (const [i] of scores.entries()) {
  ranked.push(`${i + 1}. ${scores[i]}`);
}
console.log(ranked);
