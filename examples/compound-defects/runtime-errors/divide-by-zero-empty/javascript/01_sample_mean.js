const SAMPLES = [];
function mean(samples) {
    let total = 0;
    for (const sample of samples) {
        total += sample;
    }
    return total / samples.length;
}
function main() {
    console.log("mean", mean(SAMPLES));
}
main();
