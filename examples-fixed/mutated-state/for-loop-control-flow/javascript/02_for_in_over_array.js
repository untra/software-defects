const scores = [90, 85, 77];
const ranked = [];
for (const [i] of scores.entries()) {
    ranked.push(`${i + 1}. ${scores[i]}`);
}
console.log(ranked);
