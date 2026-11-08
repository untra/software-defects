function average(scores) {
    if (scores.length === 0)
        return null;
    const total = scores.reduce((sum, score) => sum + score, 0);
    return total / scores.length;
}
const scores = [];
console.log("average", average(scores));
