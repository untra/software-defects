function average(scores: number[]): number {
  const total = scores.reduce((sum, score) => sum + score, 0);
  return total / scores.length;
}

const scores: number[] = [];
console.log("average", average(scores));
