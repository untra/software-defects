function average(scores) {
    const total = scores.reduce((sum, score) => sum + score, 0);
    return total / scores.length;
}
const scores = [];
console.log("average", average(scores));
