function average(scores: number[]): number | null {
  if (scores.length === 0) return null;
  const total = scores.reduce((sum, score) => sum + score, 0);
  return total / scores.length;
}

const scores: number[] = [];
console.log("average", average(scores));
